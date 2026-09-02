use tauri::State;

use crate::{
    app_state::AppState,
    models::{DashboardRange, DashboardSnapshot, LiveCountsSnapshot, PrivacyStatus},
};

#[tauri::command]
pub fn get_dashboard(
    range: DashboardRange,
    state: State<'_, AppState>,
) -> Result<DashboardSnapshot, String> {
    state
        .collector
        .dashboard(range)
        .map_err(|error| format!("Could not load local heatmap aggregate: {error:#}"))
}

#[tauri::command]
pub fn get_live_counts(state: State<'_, AppState>) -> LiveCountsSnapshot {
    state.collector.live_counts()
}

#[tauri::command]
pub fn set_collection_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<PrivacyStatus, String> {
    state
        .set_collection_enabled(enabled)
        .map_err(|error| format!("Could not update collection state: {error:#}"))
}

#[tauri::command]
pub fn get_privacy_status(state: State<'_, AppState>) -> PrivacyStatus {
    state.privacy_status()
}
