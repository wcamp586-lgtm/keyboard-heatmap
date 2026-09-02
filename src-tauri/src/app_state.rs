use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use anyhow::{Context, Result};
use tauri::{AppHandle, Manager, Runtime};

use crate::{
    collector::KeyCollector,
    lifecycle::{LifecycleEvent, LifecycleLogger},
    models::PrivacyStatus,
    raw_input::{self, RawInputController},
    shortcut_hook::{self, ShortcutHookController},
    storage::AggregateStore,
};

pub struct AppState {
    pub collector: Arc<KeyCollector>,
    raw_input: RawInputController,
    shortcut_hook: ShortcutHookController,
    lifecycle: LifecycleLogger,
    /// Set only by the explicit tray Exit action. It distinguishes the user's
    /// deliberate request from a generic event-loop exit request.
    explicit_exit_requested: AtomicBool,
    /// Prevents repeated tray clicks or duplicate Destroyed notifications from
    /// trying to recreate the main WebView window at the same time.
    main_window_recovery_in_progress: AtomicBool,
    /// Serializes whole Raw Input/collector state transitions so a tray action
    /// and a dashboard action cannot leave the two layers out of sync.
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

        // Avoid a brief partial-coverage interval while the paired listeners
        // start. The low-level listener is the sole source of fixed shortcut
        // aggregates, so startup fails rather than claiming coverage if its
        // Windows hook cannot be installed.
        collector.set_enabled(false);
        let raw_input = raw_input::start(Arc::clone(&collector), lifecycle.clone())
            .map_err(anyhow::Error::msg)
            .context("Could not initialize the Windows Raw Input receiver")?;
        let shortcut_hook = match shortcut_hook::start(Arc::clone(&collector), lifecycle.clone()) {
            Ok(shortcut_hook) => shortcut_hook,
            Err(error) => {
                let _ = raw_input.set_listening(false);
                return Err(anyhow::Error::msg(error)
                    .context("Could not initialize the fixed Windows shortcut listener"));
            }
        };
        collector.set_enabled(true);

        lifecycle.record(LifecycleEvent::Started);

        Ok(Self {
            collector,
            raw_input,
            shortcut_hook,
            lifecycle,
            explicit_exit_requested: AtomicBool::new(false),
            main_window_recovery_in_progress: AtomicBool::new(false),
            transition_gate: Mutex::new(()),
        })
    }

    /// Changes both layers of collection in a privacy-safe order.
    ///
    /// Pausing closes the in-memory intake gate before unregistering Raw Input,
    /// so any already queued Windows message cannot reach SQLite. Resuming
    /// performs registration first, so data cannot be accepted until Windows
    /// confirms that the private message-only window is listening again.
    pub fn set_collection_enabled(&self, enabled: bool) -> Result<PrivacyStatus> {
        let _transition = self
            .transition_gate
            .lock()
            .map_err(|_| anyhow::anyhow!("Collection-state transition lock was poisoned"))?;

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

        Ok(self.privacy_status())
    }

    pub fn privacy_status(&self) -> PrivacyStatus {
        // The existing public field remains named `raw_input_active` for API
        // compatibility, but it now represents the complete paired listener
        // stack: physical Raw Input plus the fixed-shortcut listener.
        PrivacyStatus::new(
            self.collector.is_enabled(),
            self.raw_input.is_active() && self.shortcut_hook.is_active(),
        )
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
        self.collector.set_enabled(false);
        let _ = self.raw_input.set_listening(false);
        self.shortcut_hook.shutdown();
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
}
