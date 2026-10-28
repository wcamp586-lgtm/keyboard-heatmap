//! macOS-side control and liveness probe for the standalone collection agent.
//!
//! The agent (see `src/bin/keyboard-heatmap-agent.rs`) owns the CGEventTap and
//! writes the hourly aggregates; this module only inspects its heartbeat file
//! and toggles its pause flag, so the Tauri app never needs Input Monitoring.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

/// A heartbeat older than this means the agent is not collecting (crashed,
/// paused by the system, or refused the event tap).
const HEARTBEAT_STALE_AFTER: Duration = Duration::from_secs(15);

pub struct AgentController {
    heartbeat: PathBuf,
    pause: PathBuf,
}

impl AgentController {
    pub fn new(data_directory: &Path) -> Self {
        Self {
            heartbeat: data_directory.join("agent.heartbeat"),
            pause: data_directory.join("agent.paused"),
        }
    }

    pub fn is_active(&self) -> bool {
        let Ok(metadata) = fs::metadata(&self.heartbeat) else {
            return false;
        };
        let Ok(modified) = metadata.modified() else {
            return false;
        };
        modified
            .elapsed()
            .map(|age| age < HEARTBEAT_STALE_AFTER)
            .unwrap_or(false)
    }

    pub fn is_paused(&self) -> bool {
        self.pause.exists()
    }

    pub fn set_paused(&self, paused: bool) -> Result<(), String> {
        if paused {
            fs::write(&self.pause, b"paused").map_err(|error| error.to_string())
        } else {
            match fs::remove_file(&self.pause) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error.to_string()),
            }
        }
    }

    /// Best-effort start of the LaunchAgent when the app launches and no fresh
    /// heartbeat exists yet.
    pub fn try_start(&self) {
        let Ok(output) = std::process::Command::new("id").arg("-u").output() else {
            return;
        };
        let uid = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if uid.is_empty() {
            return;
        }
        let _ = std::process::Command::new("launchctl")
            .args([
                "kickstart",
                "-k",
                &format!("gui/{uid}/com.local.keyboardheatmap.agent"),
            ])
            .output();
    }
}
