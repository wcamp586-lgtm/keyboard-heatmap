//! Standalone macOS input-collection agent.
//!
//! The Tauri app owns the UI and reads the hourly aggregates; this tiny binary
//! owns the keyboard event tap. Compiling it once and installing it outside the
//! app bundle keeps the macOS Input Monitoring grant alive across UI rebuilds.

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("keyboard-heatmap-agent is only used on macOS.");
}

#[cfg(target_os = "macos")]
fn main() {
    use std::{fs, path::PathBuf, sync::Arc, thread, time::Duration};

    use keyboard_heatmap_lib::{
        collector::KeyCollector,
        lifecycle::{LifecycleEvent, LifecycleLogger},
        raw_input,
        storage::AggregateStore,
    };

    let home = std::env::var("HOME").expect("HOME is required");
    let data_dir =
        PathBuf::from(&home).join("Library/Application Support/com.local.keyboardheatmap");
    let log_dir = PathBuf::from(&home).join("Library/Logs/com.local.keyboardheatmap");
    fs::create_dir_all(&data_dir).expect("create data directory");

    let heartbeat_path = data_dir.join("agent.heartbeat");
    let pause_path = data_dir.join("agent.paused");

    let store = Arc::new(
        AggregateStore::open(&data_dir.join("keyboard-heatmap.sqlite3"))
            .expect("open the local aggregate database"),
    );
    let collector = KeyCollector::new(store);
    let lifecycle = LifecycleLogger::open_at(&log_dir);
    let raw_input =
        raw_input::start(Arc::clone(&collector), lifecycle.clone()).expect("start the macOS tap");
    lifecycle.record(LifecycleEvent::Started);
    touch(&heartbeat_path);

    loop {
        let paused = pause_path.exists();
        if collector.is_enabled() == paused {
            collector.set_enabled(!paused);
            let _ = raw_input.set_listening(!paused);
            lifecycle.record(if paused {
                LifecycleEvent::CollectionPaused
            } else {
                LifecycleEvent::CollectionResumed
            });
        }

        // The app treats a fresh heartbeat as "the agent is collecting". When
        // the tap was refused (missing Input Monitoring) the heartbeat is
        // removed so the UI reports "unavailable" instead of guessing.
        if raw_input.is_active() {
            touch(&heartbeat_path);
        } else {
            let _ = fs::remove_file(&heartbeat_path);
        }

        thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(target_os = "macos")]
fn touch(path: &std::path::Path) {
    use std::io::Write;

    let Ok(mut file) = std::fs::File::create(path) else {
        return;
    };
    let _ = write!(file, "{}", chrono::Utc::now().timestamp());
}
