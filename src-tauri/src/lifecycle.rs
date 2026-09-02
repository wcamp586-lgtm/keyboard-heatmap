//! Minimal, privacy-safe lifecycle diagnostics.
//!
//! This log intentionally records only application lifecycle transitions. It
//! never receives keyboard data, device handles, focused-window information,
//! application names, or error strings that could carry unrelated context.

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
};

use chrono::Utc;
use tauri::{AppHandle, Manager, Runtime};

/// The only event vocabulary accepted by the local diagnostic log.
#[derive(Clone, Copy, Debug)]
pub enum LifecycleEvent {
    Started,
    WindowShown,
    WindowHidden,
    WindowRecoveryStarted,
    WindowRecoverySucceeded,
    WindowRecoveryFailed,
    CollectionPaused,
    CollectionResumed,
    ExplicitExit,
    UnintendedExitPrevented,
    RawInputStopped,
    RawInputFailed,
    ShortcutHookStarted,
    ShortcutHookStopped,
    ShortcutHookFailed,
}

impl LifecycleEvent {
    fn label(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::WindowShown => "window-shown",
            Self::WindowHidden => "window-hidden",
            Self::WindowRecoveryStarted => "window-recovery-started",
            Self::WindowRecoverySucceeded => "window-recovery-succeeded",
            Self::WindowRecoveryFailed => "window-recovery-failed",
            Self::CollectionPaused => "collection-paused",
            Self::CollectionResumed => "collection-resumed",
            Self::ExplicitExit => "explicit-exit",
            Self::UnintendedExitPrevented => "unintended-exit-prevented",
            Self::RawInputStopped => "raw-input-stopped",
            Self::RawInputFailed => "raw-input-failed",
            Self::ShortcutHookStarted => "shortcut-hook-started",
            Self::ShortcutHookStopped => "shortcut-hook-stopped",
            Self::ShortcutHookFailed => "shortcut-hook-failed",
        }
    }
}

/// Best-effort append-only lifecycle logger.
///
/// Logging must never prevent collection or make the app fail to start. If the
/// app log directory cannot be created, this instance becomes a safe no-op.
#[derive(Clone)]
pub struct LifecycleLogger {
    file: Option<Arc<Mutex<File>>>,
}

impl LifecycleLogger {
    pub fn initialize<R: Runtime>(app: &AppHandle<R>) -> Self {
        let file = app
            .path()
            .app_log_dir()
            .ok()
            .and_then(|directory| {
                fs::create_dir_all(&directory).ok()?;
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(directory.join("lifecycle.log"))
                    .ok()
            })
            .map(|file| Arc::new(Mutex::new(file)));

        Self { file }
    }

    pub fn record(&self, event: LifecycleEvent) {
        let Some(file) = &self.file else {
            return;
        };
        let Ok(mut file) = file.lock() else {
            return;
        };

        // UTC timestamps make support diagnostics ordered without recording
        // any user input or foreground-application context.
        let line = format!(
            "{} {}\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
            event.label()
        );
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}
