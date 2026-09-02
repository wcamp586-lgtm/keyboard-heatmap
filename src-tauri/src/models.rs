use serde::{Deserialize, Serialize};

/// Time window accepted by the `get_dashboard` Tauri command.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DashboardRange {
    Today,
    Week,
    Month,
    All,
}

impl Default for DashboardRange {
    fn default() -> Self {
        Self::Today
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyCount {
    pub key_id: String,
    pub count: u64,
}

/// One built-in, allowlisted shortcut aggregate. `shortcut_id` is generated
/// from a fixed numeric identity after SQLite is read; arbitrary chords are
/// neither accepted nor persisted.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutCount {
    pub shortcut_id: String,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyCount {
    /// Local hour-of-day (0 through 23), derived only from the persisted hour
    /// bucket when a dashboard is requested.
    pub hour: u8,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSnapshot {
    pub range: DashboardRange,
    pub range_start_utc: i64,
    pub range_end_utc: i64,
    pub total_presses: u64,
    pub unique_keys: u32,
    /// Count of recognised, built-in Ctrl, Alt, and Win shortcuts. This is
    /// separate from physical key totals, so a recognised combination adds
    /// one shortcut aggregate without changing the heatmap's semantics.
    pub total_shortcuts: u64,
    /// Always contains all 104 fixed layout positions, including zero-count
    /// keys, so the frontend can render the complete keyboard consistently.
    pub key_counts: Vec<KeyCount>,
    pub top_keys: Vec<KeyCount>,
    /// At most ten non-zero fixed shortcut aggregates, ranked by use count.
    pub top_shortcuts: Vec<ShortcutCount>,
    /// Always contains 24 local-hour bins.
    pub hourly_distribution: Vec<HourlyCount>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveCountsSnapshot {
    pub collection_enabled: bool,
    pub session_total_presses: u64,
    /// Non-zero session counts only. Historical counts belong in
    /// `get_dashboard`, which is based on SQLite aggregates.
    pub key_counts: Vec<KeyCount>,
    /// Non-zero session counts for built-in, allowlisted shortcuts only.
    pub session_total_shortcuts: u64,
    pub shortcut_counts: Vec<ShortcutCount>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyStatus {
    pub collection_enabled: bool,
    /// Backward-compatible name for the complete Windows listener stack:
    /// physical Raw Input plus the fixed-shortcut listener.
    pub raw_input_active: bool,
    /// `collecting`, `paused`, or `unavailable`. This reflects collection
    /// health only and never contains input or process metadata.
    pub collection_health: &'static str,
    pub keyboard_layout: &'static str,
    pub storage_mode: &'static str,
    /// Union of the fixed columns used by the two aggregate-only tables.
    pub persisted_fields: [&'static str; 4],
    pub never_collected: [&'static str; 6],
}

impl PrivacyStatus {
    pub fn new(collection_enabled: bool, raw_input_active: bool) -> Self {
        Self {
            collection_enabled,
            raw_input_active,
            collection_health: if !collection_enabled {
                "paused"
            } else if raw_input_active {
                "collecting"
            } else {
                "unavailable"
            },
            keyboard_layout: "ansi-104-physical",
            storage_mode: "local-hourly-aggregate-only",
            persisted_fields: ["hour_start_utc", "key_id", "shortcut_id", "press_count"],
            never_collected: [
                "typed_characters",
                "key_order",
                "clipboard",
                "window_titles",
                "application_names",
                "device_identifiers",
            ],
        }
    }
}
