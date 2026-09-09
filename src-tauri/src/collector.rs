//! In-memory intake and batched persistence for fixed-key aggregates.
//!
//! Raw Input can only hand this type a fixed `KeyId` and a volatile opaque
//! device handle for repeat suppression. The Windows low-level shortcut hook
//! can only hand it a fixed `ShortcutId`. The writer receives only an hour
//! bucket plus one fixed ID at a time; it never sees a raw event, a character,
//! a key sequence, or a device.

use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{anyhow, Result};
use chrono::Utc;

use crate::{
    date_range::DateSelection,
    keymap::KeyId,
    models::{
        DashboardRange, DashboardSnapshot, DayDetail, HistoryPage, HistoryTarget, KeyCount,
        LiveCountsSnapshot, ShortcutCount, StorageHealthSnapshot,
    },
    shortcuts::ShortcutId,
    storage::{utc_hour_start, AggregateStore, HourlyKeyBatch, HourlyShortcutBatch},
};

#[derive(Clone, Copy)]
struct AggregatePress {
    hour_start_utc: i64,
    key_id: KeyId,
}

enum WriterMessage {
    Press(AggregatePress),
    Shortcut {
        hour_start_utc: i64,
        shortcut_id: ShortcutId,
    },
    Flush(Sender<Result<(), String>>),
    Shutdown(Sender<()>),
}

#[derive(Debug)]
struct StorageHealthState {
    status: &'static str,
    last_successful_flush_at: Option<i64>,
    last_error_code: Option<&'static str>,
    consecutive_failures: u32,
}

impl Default for StorageHealthState {
    fn default() -> Self {
        Self {
            status: "healthy",
            last_successful_flush_at: None,
            last_error_code: None,
            consecutive_failures: 0,
        }
    }
}

#[derive(Default)]
struct StorageHealth {
    state: Mutex<StorageHealthState>,
}

impl StorageHealth {
    fn record_success(&self) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.status = "healthy";
        state.last_successful_flush_at = Some(Utc::now().timestamp());
        state.last_error_code = None;
        state.consecutive_failures = 0;
    }

    fn record_failure(&self, status: &'static str, error_code: &'static str) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.status = status;
        state.last_error_code = Some(error_code);
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
    }

    fn snapshot(&self) -> StorageHealthSnapshot {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        StorageHealthSnapshot {
            status: state.status,
            last_successful_flush_at: state.last_successful_flush_at,
            last_error_code: state.last_error_code,
            consecutive_failures: state.consecutive_failures,
            storage_mode: "local-hourly-aggregate-only",
        }
    }
}

/// Coordinates the Raw Input receiver and the private local aggregate store.
pub struct KeyCollector {
    enabled: AtomicBool,
    shutdown_requested: AtomicBool,
    /// Serializes the enable gate with intake so pausing cannot race an already
    /// observed Raw Input message into the persistence queue.
    intake_gate: Mutex<()>,
    live_counts: Mutex<BTreeMap<KeyId, u64>>,
    live_shortcut_counts: Mutex<BTreeMap<ShortcutId, u64>>,
    /// Opaque device handles are only used here to stop OS key-repeat messages
    /// becoming multiple presses. This set is never serialized or exposed.
    held_keys: Mutex<HashSet<(usize, KeyId)>>,
    writer: Sender<WriterMessage>,
    writer_thread: Mutex<Option<JoinHandle<()>>>,
    storage_health: Arc<StorageHealth>,
    store: Arc<AggregateStore>,
}

impl KeyCollector {
    pub fn new(store: Arc<AggregateStore>) -> Arc<Self> {
        let (writer, receiver) = mpsc::channel();
        let writer_store = Arc::clone(&store);
        let storage_health = Arc::new(StorageHealth::default());
        let writer_health = Arc::clone(&storage_health);
        let writer_thread = thread::Builder::new()
            .name("keyboard-heatmap-sqlite-writer".to_owned())
            .spawn(move || writer_loop(writer_store, receiver, writer_health))
            .expect("Could not start local aggregate writer thread");

        Arc::new(Self {
            enabled: AtomicBool::new(true),
            shutdown_requested: AtomicBool::new(false),
            intake_gate: Mutex::new(()),
            live_counts: Mutex::new(BTreeMap::new()),
            live_shortcut_counts: Mutex::new(BTreeMap::new()),
            held_keys: Mutex::new(HashSet::new()),
            writer,
            writer_thread: Mutex::new(Some(writer_thread)),
            storage_health,
            store,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// This value is intentionally memory-only. Persisting it would violate the
    /// fixed aggregate-table data model.
    pub fn set_enabled(&self, enabled: bool) {
        let _gate = self.intake_gate.lock().expect("Intake gate lock poisoned");
        self.enabled.store(enabled, Ordering::Release);
        // A key could be released while paused, so retaining held entries would
        // cause the next press after resume to be mistaken for an auto-repeat.
        self.held_keys
            .lock()
            .expect("Held key lock poisoned")
            .clear();
    }

    /// Handles a raw make event after the native receiver has mapped the scan
    /// position. `device_id` is an opaque, process-local handle only.
    pub fn handle_raw_make(&self, key_id: KeyId, device_id: usize) {
        self.handle_raw_make_at(key_id, device_id, utc_hour_start(Utc::now().timestamp()));
    }

    fn handle_raw_make_at(&self, key_id: KeyId, device_id: usize, hour_start_utc: i64) {
        let _gate = self.intake_gate.lock().expect("Intake gate lock poisoned");
        if self.shutdown_requested.load(Ordering::Acquire) || !self.is_enabled() {
            return;
        }

        // Pause commonly has no separate break event. Count each dedicated
        // Pause make directly instead of permanently retaining it as "held".
        if !key_id.does_not_emit_break() {
            let mut held_keys = self.held_keys.lock().expect("Held key lock poisoned");
            if !held_keys.insert((device_id, key_id)) {
                return;
            }
        }

        // Raw Input owns physical heatmap positions only. Fixed shortcut
        // aggregates are generated exclusively by the low-level hook, which
        // is reliable for Windows-reserved combinations such as Win+D and
        // Alt+Tab and prevents cross-listener double counting.
        self.enqueue_press(key_id, hour_start_utc);
    }

    /// Handles a raw break event. It changes only volatile repeat-suppression
    /// state and does not increment a count or reach SQLite.
    pub fn handle_raw_break(&self, key_id: KeyId, device_id: usize) {
        self.held_keys
            .lock()
            .expect("Held key lock poisoned")
            .remove(&(device_id, key_id));
    }

    /// Called when Windows announces that a raw-input device disappeared. The
    /// opaque device value is immediately removed from volatile state.
    pub fn forget_device(&self, device_id: usize) {
        self.held_keys
            .lock()
            .expect("Held key lock poisoned")
            .retain(|(held_device, _)| *held_device != device_id);
    }

    /// Accepts one already-recognized fixed shortcut aggregate from the
    /// dedicated low-level hook message window. This method receives no
    /// virtual-key code, scan code, modifier sequence, window title, app, or
    /// device information, and it never changes physical heatmap counts.
    pub fn handle_shortcut_make(&self, shortcut_id: ShortcutId) {
        self.handle_shortcut_make_at(shortcut_id, utc_hour_start(Utc::now().timestamp()));
    }

    fn handle_shortcut_make_at(&self, shortcut_id: ShortcutId, hour_start_utc: i64) {
        let _gate = self.intake_gate.lock().expect("Intake gate lock poisoned");
        if self.shutdown_requested.load(Ordering::Acquire) || !self.is_enabled() {
            return;
        }

        if self
            .writer
            .send(WriterMessage::Shortcut {
                hour_start_utc,
                shortcut_id,
            })
            .is_err()
        {
            self.storage_health
                .record_failure("unavailable", "writer_unavailable");
            return;
        }

        let mut live_shortcut_counts = self
            .live_shortcut_counts
            .lock()
            .expect("Live shortcut count lock poisoned");
        *live_shortcut_counts.entry(shortcut_id).or_default() += 1;
    }

    fn enqueue_press(&self, key_id: KeyId, hour_start_utc: i64) {
        if self
            .writer
            .send(WriterMessage::Press(AggregatePress {
                hour_start_utc,
                key_id,
            }))
            .is_err()
        {
            self.storage_health
                .record_failure("unavailable", "writer_unavailable");
            return;
        }

        let mut live_counts = self.live_counts.lock().expect("Live count lock poisoned");
        *live_counts.entry(key_id).or_default() += 1;
    }

    pub fn flush(&self) -> Result<()> {
        let (reply, receive) = mpsc::channel();
        self.writer.send(WriterMessage::Flush(reply)).map_err(|_| {
            self.storage_health
                .record_failure("unavailable", "writer_unavailable");
            anyhow!("Local aggregate writer is unavailable")
        })?;
        match receive.recv_timeout(Duration::from_secs(2)) {
            Ok(result) => result.map_err(|message| anyhow!(message)),
            Err(_) => {
                self.storage_health
                    .record_failure("degraded", "writer_timeout");
                Err(anyhow!("Timed out waiting for local aggregate writer"))
            }
        }
    }

    pub fn dashboard(&self, range: DashboardRange) -> Result<DashboardSnapshot> {
        // A failed write must be visible through `storage_health`, but an
        // otherwise readable historical dashboard is still useful.
        let _ = self.flush();
        self.store.dashboard(range)
    }

    pub fn storage_health(&self) -> StorageHealthSnapshot {
        self.storage_health.snapshot()
    }

    pub fn dashboard_selection(&self, selection: &DateSelection) -> Result<DashboardSnapshot> {
        let _ = self.flush();
        self.store.dashboard_selection(selection)
    }

    pub fn history(
        &self,
        selection: &DateSelection,
        target: Option<&HistoryTarget>,
        page: u32,
    ) -> Result<HistoryPage> {
        let _ = self.flush();
        self.store.history(selection, target, page)
    }

    pub fn day_detail(&self, date: &str, target: Option<&HistoryTarget>) -> Result<DayDetail> {
        let _ = self.flush();
        self.store.day_detail(date, target)
    }

    pub fn live_counts(&self) -> LiveCountsSnapshot {
        let counts = self.live_counts.lock().expect("Live count lock poisoned");
        let session_total_presses = counts.values().sum();
        let key_counts = counts
            .iter()
            .map(|(key_id, count)| KeyCount {
                key_id: key_id.wire_id().to_owned(),
                count: *count,
            })
            .collect();
        let shortcut_counts = self
            .live_shortcut_counts
            .lock()
            .expect("Live shortcut count lock poisoned");
        let session_total_shortcuts = shortcut_counts.values().sum();
        let shortcut_counts = shortcut_counts
            .iter()
            .map(|(shortcut_id, count)| ShortcutCount {
                shortcut_id: shortcut_id.wire_id().to_owned(),
                count: *count,
            })
            .collect();

        LiveCountsSnapshot {
            collection_enabled: self.is_enabled(),
            session_total_presses,
            key_counts,
            session_total_shortcuts,
            shortcut_counts,
        }
    }

    pub fn shutdown(&self) {
        if self.shutdown_requested.swap(true, Ordering::AcqRel) {
            return;
        }
        self.set_enabled(false);

        let (reply, receive) = mpsc::channel();
        let _ = self.writer.send(WriterMessage::Shutdown(reply));
        let acknowledged = receive.recv_timeout(Duration::from_secs(2)).is_ok();
        let thread = self
            .writer_thread
            .lock()
            .expect("Writer thread lock poisoned")
            .take();
        if acknowledged {
            if let Some(thread) = thread {
                let _ = thread.join();
            }
        } else {
            self.storage_health
                .record_failure("unavailable", "writer_shutdown_timeout");
            // Dropping a JoinHandle detaches it, allowing the process to exit
            // instead of blocking forever after the bounded acknowledgement.
            drop(thread);
        }
    }
}

fn writer_loop(
    store: Arc<AggregateStore>,
    receiver: Receiver<WriterMessage>,
    health: Arc<StorageHealth>,
) {
    let mut pending_keys = HourlyKeyBatch::new();
    let mut pending_shortcuts = HourlyShortcutBatch::new();
    loop {
        match receiver.recv_timeout(Duration::from_millis(250)) {
            Ok(WriterMessage::Press(press)) => {
                *pending_keys
                    .entry((press.hour_start_utc, press.key_id))
                    .or_default() += 1;
                // Preserve bounded memory even during uninterrupted typing.
                if pending_keys.len() + pending_shortcuts.len() >= 64 {
                    let _ = flush_pending_and_record(
                        &store,
                        &mut pending_keys,
                        &mut pending_shortcuts,
                        &health,
                    );
                }
            }
            Ok(WriterMessage::Shortcut {
                hour_start_utc,
                shortcut_id,
            }) => {
                *pending_shortcuts
                    .entry((hour_start_utc, shortcut_id))
                    .or_default() += 1;
                // Preserve bounded memory even during uninterrupted typing.
                if pending_keys.len() + pending_shortcuts.len() >= 64 {
                    let _ = flush_pending_and_record(
                        &store,
                        &mut pending_keys,
                        &mut pending_shortcuts,
                        &health,
                    );
                }
            }
            Ok(WriterMessage::Flush(reply)) => {
                let result = flush_pending_and_record(
                    &store,
                    &mut pending_keys,
                    &mut pending_shortcuts,
                    &health,
                )
                .map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            Ok(WriterMessage::Shutdown(reply)) => {
                let _ = flush_pending_and_record(
                    &store,
                    &mut pending_keys,
                    &mut pending_shortcuts,
                    &health,
                );
                let _ = reply.send(());
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = flush_pending_and_record(
                    &store,
                    &mut pending_keys,
                    &mut pending_shortcuts,
                    &health,
                );
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = flush_pending_and_record(
                    &store,
                    &mut pending_keys,
                    &mut pending_shortcuts,
                    &health,
                );
                break;
            }
        }
    }
}

fn flush_pending_and_record(
    store: &AggregateStore,
    pending_keys: &mut HourlyKeyBatch,
    pending_shortcuts: &mut HourlyShortcutBatch,
    health: &StorageHealth,
) -> Result<()> {
    let had_pending = !pending_keys.is_empty() || !pending_shortcuts.is_empty();
    match flush_pending(store, pending_keys, pending_shortcuts) {
        Ok(()) => {
            if had_pending {
                health.record_success();
            }
            Ok(())
        }
        Err(error) => {
            health.record_failure("degraded", "sqlite_write_failed");
            Err(error)
        }
    }
}

fn flush_pending(
    store: &AggregateStore,
    pending_keys: &mut HourlyKeyBatch,
    pending_shortcuts: &mut HourlyShortcutBatch,
) -> Result<()> {
    if pending_keys.is_empty() && pending_shortcuts.is_empty() {
        return Ok(());
    }
    store.increment_batch(pending_keys, pending_shortcuts)?;
    pending_keys.clear();
    pending_shortcuts.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::KeyCollector;
    use crate::{
        keymap::KeyId, models::DashboardRange, shortcuts::ShortcutId, storage::AggregateStore,
    };

    #[test]
    fn make_repeat_break_make_counts_two_real_presses() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::KeyA, 1, 3_600);
        collector.handle_raw_make_at(KeyId::KeyA, 1, 3_600);
        collector.handle_raw_break(KeyId::KeyA, 1);
        collector.handle_raw_make_at(KeyId::KeyA, 1, 3_600);
        collector.flush().unwrap();

        assert_eq!(collector.live_counts().session_total_presses, 2);
        assert_eq!(
            collector
                .store
                .dashboard_between(DashboardRange::All, 0, 7_200)
                .unwrap()
                .total_presses,
            2
        );
        collector.shutdown();
    }

    #[test]
    fn two_devices_can_press_the_same_physical_key_once_each() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::KeyA, 100, 3_600);
        collector.handle_raw_make_at(KeyId::KeyA, 200, 3_600);
        collector.flush().unwrap();

        assert_eq!(collector.live_counts().session_total_presses, 2);
        collector.shutdown();
    }

    #[test]
    fn ctrl_c_counts_both_physical_keys_and_one_fixed_shortcut() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::ControlLeft, 1, 3_600);
        collector.handle_raw_make_at(KeyId::KeyC, 1, 3_600);
        collector.handle_raw_break(KeyId::KeyC, 1);
        collector.handle_raw_break(KeyId::ControlLeft, 1);
        collector.handle_shortcut_make_at(ShortcutId::CtrlC, 3_600);
        collector.flush().unwrap();

        let dashboard = collector
            .store
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_presses, 2);
        assert_eq!(dashboard.total_shortcuts, 1);
        assert_eq!(dashboard.top_shortcuts[0].shortcut_id, "ctrl_c");
        assert_eq!(dashboard.top_shortcuts[0].count, 1);
        assert_eq!(
            dashboard
                .key_counts
                .iter()
                .find(|count| count.key_id == "control_left")
                .unwrap()
                .count,
            1
        );
        assert_eq!(
            dashboard
                .key_counts
                .iter()
                .find(|count| count.key_id == "key_c")
                .unwrap()
                .count,
            1
        );
        assert_eq!(collector.live_counts().session_total_shortcuts, 1);
        collector.shutdown();
    }

    #[test]
    fn shortcut_key_auto_repeat_does_not_duplicate_the_shortcut_aggregate() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::ControlRight, 1, 3_600);
        collector.handle_raw_make_at(KeyId::KeyV, 1, 3_600);
        collector.handle_raw_make_at(KeyId::KeyV, 1, 3_600);
        collector.handle_raw_make_at(KeyId::KeyV, 1, 3_600);
        collector.handle_shortcut_make_at(ShortcutId::CtrlV, 3_600);
        collector.flush().unwrap();

        let dashboard = collector
            .store
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_presses, 2);
        assert_eq!(dashboard.total_shortcuts, 1);
        assert_eq!(dashboard.top_shortcuts[0].shortcut_id, "ctrl_v");
        collector.shutdown();
    }

    #[test]
    fn wrong_key_order_never_backfills_a_shortcut() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::KeyC, 1, 3_600);
        collector.handle_raw_make_at(KeyId::ControlLeft, 1, 3_600);
        collector.flush().unwrap();

        let dashboard = collector
            .store
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_presses, 2);
        assert_eq!(dashboard.total_shortcuts, 0);
        assert!(dashboard.top_shortcuts.is_empty());
        collector.shutdown();
    }

    #[test]
    fn modifier_from_a_different_raw_input_device_does_not_form_a_shortcut() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::ControlLeft, 100, 3_600);
        collector.handle_raw_make_at(KeyId::KeyC, 200, 3_600);
        collector.flush().unwrap();

        let dashboard = collector
            .store
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_presses, 2);
        assert_eq!(dashboard.total_shortcuts, 0);
        collector.shutdown();
    }

    #[test]
    fn win_d_alt_tab_and_alt_f4_are_each_aggregated_once() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::MetaLeft, 1, 3_600);
        collector.handle_raw_make_at(KeyId::KeyD, 1, 3_600);
        // Repeated make messages while D remains held cannot create another
        // physical-key or shortcut count.
        collector.handle_raw_make_at(KeyId::KeyD, 1, 3_600);
        collector.handle_raw_break(KeyId::KeyD, 1);
        collector.handle_raw_break(KeyId::MetaLeft, 1);

        collector.handle_raw_make_at(KeyId::AltLeft, 1, 3_600);
        collector.handle_raw_make_at(KeyId::Tab, 1, 3_600);
        collector.handle_raw_break(KeyId::Tab, 1);
        collector.handle_raw_break(KeyId::AltLeft, 1);

        collector.handle_raw_make_at(KeyId::AltRight, 1, 3_600);
        collector.handle_raw_make_at(KeyId::F4, 1, 3_600);
        collector.handle_raw_break(KeyId::F4, 1);
        collector.handle_raw_break(KeyId::AltRight, 1);
        collector.handle_shortcut_make_at(ShortcutId::WinD, 3_600);
        collector.handle_shortcut_make_at(ShortcutId::AltTab, 3_600);
        collector.handle_shortcut_make_at(ShortcutId::AltF4, 3_600);
        collector.flush().unwrap();

        let dashboard = collector
            .store
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_presses, 6);
        assert_eq!(dashboard.total_shortcuts, 3);
        for expected_shortcut in ["win_d", "alt_tab", "alt_f4"] {
            assert_eq!(
                dashboard
                    .top_shortcuts
                    .iter()
                    .find(|count| count.shortcut_id == expected_shortcut)
                    .unwrap()
                    .count,
                1
            );
        }
        collector.shutdown();
    }

    #[test]
    fn shortcut_message_changes_only_the_fixed_shortcut_aggregate() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_shortcut_make_at(ShortcutId::WinD, 3_600);
        collector.flush().unwrap();

        let dashboard = collector
            .store
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_presses, 0);
        assert_eq!(dashboard.total_shortcuts, 1);
        assert_eq!(dashboard.top_shortcuts[0].shortcut_id, "win_d");
        assert_eq!(collector.live_counts().session_total_presses, 0);
        assert_eq!(collector.live_counts().session_total_shortcuts, 1);
        collector.shutdown();
    }

    #[test]
    fn pause_gate_drops_new_messages_and_resets_held_state() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        collector.handle_raw_make_at(KeyId::KeyW, 1, 3_600);
        collector.set_enabled(false);
        collector.handle_raw_make_at(KeyId::KeyW, 1, 3_600);
        collector.set_enabled(true);
        collector.handle_raw_make_at(KeyId::KeyW, 1, 3_600);
        collector.flush().unwrap();

        assert_eq!(collector.live_counts().session_total_presses, 2);
        collector.shutdown();
    }

    #[test]
    fn successful_flush_updates_storage_health_without_exposing_input() {
        let store = Arc::new(AggregateStore::open_in_memory().unwrap());
        let collector = KeyCollector::new(store);

        assert_eq!(collector.storage_health().status, "healthy");
        assert_eq!(collector.storage_health().last_successful_flush_at, None);

        collector.handle_raw_make_at(KeyId::KeyA, 1, 3_600);
        collector.flush().unwrap();

        let health = collector.storage_health();
        assert_eq!(health.status, "healthy");
        assert!(health.last_successful_flush_at.is_some());
        assert_eq!(health.last_error_code, None);
        assert_eq!(health.consecutive_failures, 0);
        collector.shutdown();
    }
}
