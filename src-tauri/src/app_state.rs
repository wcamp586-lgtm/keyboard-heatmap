use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use anyhow::{Context, Result};
use chrono::Utc;
use tauri::{AppHandle, Manager, Runtime};

use crate::{
    collector::KeyCollector,
    date_range::DateSelection,
    lifecycle::{LifecycleEvent, LifecycleLogger},
    models::{AppSnapshot, CollectionStatus, PrivacyStatus},
    storage::AggregateStore,
};

#[cfg(target_os = "macos")]
use crate::agent_controller::AgentController;

#[cfg(not(target_os = "macos"))]
use crate::{
    raw_input::{self, RawInputController},
    shortcut_hook::{self, ShortcutHookController},
};

pub struct AppState {
    pub collector: Arc<KeyCollector>,
    #[cfg(not(target_os = "macos"))]
    raw_input: RawInputController,
    #[cfg(not(target_os = "macos"))]
    shortcut_hook: ShortcutHookController,
    /// On macOS the keyboard listener lives in a separately installed agent so
    /// rebuilding the app does not invalidate the Input Monitoring grant.
    #[cfg(target_os = "macos")]
    agent: AgentController,
    lifecycle: LifecycleLogger,
    /// Set only by the explicit tray Exit action. It distinguishes the user's
    /// deliberate request from a generic event-loop exit request.
    explicit_exit_requested: AtomicBool,
    /// Prevents repeated tray clicks or duplicate Destroyed notifications from
    /// trying to recreate the main WebView window at the same time.
    main_window_recovery_in_progress: AtomicBool,
    /// Serializes whole listener/collector state transitions so a tray action
    /// and a dashboard action cannot leave the layers out of sync.
    transition_gate: Mutex<()>,
}

impl AppState {
    pub fn initialize<R: Runtime>(app: &AppHandle<R>) -> Result<Self> {
        let lifecycle = LifecycleLogger::initialize(app);
        let data_directory = app
            .path()
            .app_data_dir()
            .context("Could not resolve the private local application-data directory")?;
        let database_path = data_directory.join("keyboard-heatmap.sqlite3");
        let store = Arc::new(AggregateStore::open(&database_path)?);
        let collector = KeyCollector::new(store);

        collector.set_enabled(false);

        #[cfg(target_os = "macos")]
        let agent = {
            let agent = AgentController::new(&data_directory);
            if !agent.is_active() {
                agent.try_start();
            }
            agent
        };

        #[cfg(not(target_os = "macos"))]
        let (raw_input, shortcut_hook) = {
            // Avoid a brief partial-coverage interval while the paired listeners
            // start. The low-level listener is the sole source of fixed shortcut
            // aggregates, so startup fails rather than claiming coverage if its
            // hook cannot be installed.
            let raw_input = raw_input::start(Arc::clone(&collector), lifecycle.clone())
                .map_err(anyhow::Error::msg)
                .context("Could not initialize the Windows Raw Input receiver")?;
            let shortcut_hook =
                match shortcut_hook::start(Arc::clone(&collector), lifecycle.clone()) {
                    Ok(shortcut_hook) => shortcut_hook,
                    Err(error) => {
                        let _ = raw_input.set_listening(false);
                        return Err(anyhow::Error::msg(error)
                            .context("Could not initialize the fixed Windows shortcut listener"));
                    }
                };
            (raw_input, shortcut_hook)
        };

        collector.set_enabled(true);

        lifecycle.record(LifecycleEvent::Started);

        Ok(Self {
            collector,
            #[cfg(not(target_os = "macos"))]
            raw_input,
            #[cfg(not(target_os = "macos"))]
            shortcut_hook,
            #[cfg(target_os = "macos")]
            agent,
            lifecycle,
            explicit_exit_requested: AtomicBool::new(false),
            main_window_recovery_in_progress: AtomicBool::new(false),
            transition_gate: Mutex::new(()),
        })
    }

    /// Changes the collection layers in a privacy-safe order.
    ///
    /// Pausing closes the in-memory intake gate before stopping the listener so
    /// an already queued event cannot reach SQLite. Resuming re-enables the
    /// listener first, then opens the gate.
    pub fn set_collection_enabled(&self, enabled: bool) -> Result<PrivacyStatus> {
        let _transition = self
            .transition_gate
            .lock()
            .map_err(|_| anyhow::anyhow!("Collection-state transition lock was poisoned"))?;

        #[cfg(target_os = "macos")]
        {
            self.agent
                .set_paused(!enabled)
                .map_err(anyhow::Error::msg)
                .context("Could not change the collection agent state")?;
            self.lifecycle.record(if enabled {
                LifecycleEvent::CollectionResumed
            } else {
                LifecycleEvent::CollectionPaused
            });
        }

        #[cfg(not(target_os = "macos"))]
        {
            if enabled {
                self.raw_input
                    .set_listening(true)
                    .map_err(anyhow::Error::msg)
                    .context("Could not resume Windows Raw Input")?;
                if let Err(error) = self.shortcut_hook.set_listening(true) {
                    let _ = self.raw_input.set_listening(false);
                    return Err(anyhow::Error::msg(error)
                        .context("Could not resume the fixed Windows shortcut listener"));
                }
                self.collector.set_enabled(true);
                self.lifecycle.record(LifecycleEvent::CollectionResumed);
            } else {
                self.collector.set_enabled(false);
                let raw_result = self.raw_input.set_listening(false);
                let shortcut_result = self.shortcut_hook.set_listening(false);
                raw_result
                    .map_err(anyhow::Error::msg)
                    .context("Could not pause Windows Raw Input")?;
                shortcut_result
                    .map_err(anyhow::Error::msg)
                    .context("Could not pause the fixed Windows shortcut listener")?;
                self.lifecycle.record(LifecycleEvent::CollectionPaused);
            }
        }

        Ok(self.privacy_status())
    }

    /// Whether the user has collection enabled (as opposed to paused).
    fn collection_enabled(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            !self.agent.is_paused()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.collector.is_enabled()
        }
    }

    /// `(physical_listener_active, shortcut_listener_active)`. macOS reports
    /// both from the same agent heartbeat because one event tap powers both.
    fn listener_states(&self) -> (bool, bool) {
        #[cfg(target_os = "macos")]
        {
            let active = self.agent.is_active();
            (active, active)
        }
        #[cfg(not(target_os = "macos"))]
        {
            (self.raw_input.is_active(), self.shortcut_hook.is_active())
        }
    }

    pub fn privacy_status(&self) -> PrivacyStatus {
        // The existing public field remains named `raw_input_active` for API
        // compatibility, but it now represents the complete listener stack.
        let (physical_active, shortcut_active) = self.listener_states();
        PrivacyStatus::new(
            self.collection_enabled(),
            physical_active && shortcut_active,
        )
    }

    pub fn collection_status(&self) -> CollectionStatus {
        let enabled = self.collection_enabled();
        let (physical_active, shortcut_active) = self.listener_states();
        CollectionStatus {
            enabled,
            overall: if !enabled {
                "paused"
            } else if physical_active && shortcut_active {
                "collecting"
            } else if physical_active || shortcut_active {
                "degraded"
            } else {
                "unavailable"
            },
            physical_listener: listener_status(enabled, physical_active),
            shortcut_listener: listener_status(enabled, shortcut_active),
        }
    }

    pub fn app_snapshot(&self, selection: &DateSelection) -> Result<AppSnapshot> {
        let dashboard = self.collector.dashboard_selection(selection)?;
        Ok(AppSnapshot {
            generated_at: Utc::now().timestamp(),
            dashboard,
            collection: self.collection_status(),
            storage: self.collector.storage_health(),
        })
    }

    pub fn note_window_shown(&self) {
        self.lifecycle.record(LifecycleEvent::WindowShown);
    }

    pub fn note_window_hidden(&self) {
        self.lifecycle.record(LifecycleEvent::WindowHidden);
    }

    pub fn begin_main_window_recovery(&self) -> bool {
        if self.explicit_exit_requested() {
            return false;
        }

        if self
            .main_window_recovery_in_progress
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.lifecycle.record(LifecycleEvent::WindowRecoveryStarted);
            true
        } else {
            false
        }
    }

    pub fn finish_main_window_recovery(&self, succeeded: bool) {
        self.main_window_recovery_in_progress
            .store(false, Ordering::Release);
        self.lifecycle.record(if succeeded {
            LifecycleEvent::WindowRecoverySucceeded
        } else {
            LifecycleEvent::WindowRecoveryFailed
        });
    }

    pub fn prepare_for_explicit_exit(&self) {
        self.explicit_exit_requested.store(true, Ordering::Release);
        self.lifecycle.record(LifecycleEvent::ExplicitExit);
        // On macOS the collection agent is intentionally independent: quitting
        // the dashboard must not stop local aggregation.
        #[cfg(not(target_os = "macos"))]
        {
            self.collector.set_enabled(false);
            let _ = self.raw_input.set_listening(false);
            self.shortcut_hook.shutdown();
        }
        // Flush only aggregates that were already accepted. No raw input,
        // device state, or text is written during shutdown.
        self.collector.shutdown();
    }

    pub fn explicit_exit_requested(&self) -> bool {
        self.explicit_exit_requested.load(Ordering::Acquire)
    }

    pub fn note_unintended_exit_prevented(&self) {
        self.lifecycle
            .record(LifecycleEvent::UnintendedExitPrevented);
    }

    pub fn note_second_instance_redirected(&self) {
        self.lifecycle
            .record(LifecycleEvent::SecondInstanceRedirected);
    }
}

fn listener_status(enabled: bool, active: bool) -> &'static str {
    if !enabled {
        "paused"
    } else if active {
        "active"
    } else {
        "unavailable"
    }
}
