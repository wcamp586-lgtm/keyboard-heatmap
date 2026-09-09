use tauri::{AppHandle, Manager, State};

use crate::{
    app_state::AppState,
    date_range::DateSelection,
    models::{
        AppSnapshot, DashboardRange, DashboardSnapshot, DayDetail, HistoryPage, HistoryTarget,
        LiveCountsSnapshot, PrivacyStatus,
    },
    tray,
};

#[tauri::command]
pub async fn get_app_snapshot(
    range: DashboardRange,
    start_date: Option<String>,
    end_date: Option<String>,
    app: AppHandle,
) -> Result<AppSnapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<AppState>()
            .app_snapshot(&DateSelection {
                range,
                start_date,
                end_date,
            })
            .map_err(|_| "无法读取统计，请检查日期范围后重试".to_owned())
    })
    .await
    .map_err(|_| "统计读取任务未完成".to_owned())?
}

#[tauri::command]
pub async fn get_history(
    range: DashboardRange,
    start_date: Option<String>,
    end_date: Option<String>,
    target: Option<HistoryTarget>,
    page: u32,
    app: AppHandle,
) -> Result<HistoryPage, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<AppState>()
            .collector
            .history(
                &DateSelection {
                    range,
                    start_date,
                    end_date,
                },
                target.as_ref(),
                page,
            )
            .map_err(|_| "无法读取历史记录，请检查日期范围后重试".to_owned())
    })
    .await
    .map_err(|_| "历史记录读取任务未完成".to_owned())?
}

#[tauri::command]
pub async fn get_day_detail(
    date: String,
    target: Option<HistoryTarget>,
    app: AppHandle,
) -> Result<DayDetail, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<AppState>()
            .collector
            .day_detail(&date, target.as_ref())
            .map_err(|_| "无法读取小时记录，请检查日期后重试".to_owned())
    })
    .await
    .map_err(|_| "小时记录读取任务未完成".to_owned())?
}

#[tauri::command]
pub fn get_dashboard(
    range: DashboardRange,
    state: State<'_, AppState>,
) -> Result<DashboardSnapshot, String> {
    state
        .collector
        .dashboard(range)
        .map_err(|_| "Could not load the local heatmap aggregate".to_owned())
}

#[tauri::command]
pub fn get_live_counts(state: State<'_, AppState>) -> LiveCountsSnapshot {
    state.collector.live_counts()
}

#[tauri::command]
pub fn set_collection_enabled(
    enabled: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<PrivacyStatus, String> {
    let status = state
        .set_collection_enabled(enabled)
        .map_err(|_| "Could not update collection state".to_owned())?;
    tray::sync_status(&app);
    Ok(status)
}

#[tauri::command]
pub fn get_privacy_status(state: State<'_, AppState>) -> PrivacyStatus {
    state.privacy_status()
}
