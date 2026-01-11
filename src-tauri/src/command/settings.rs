use crate::app::AppState;
use crate::core::{GlobalSettings, OperationResult};
use tauri::State;

/// 获取全局设置
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> GlobalSettings {
    state.get_settings()
}

/// 更新全局设置
#[tauri::command]
pub fn update_settings(state: State<AppState>, settings: GlobalSettings) -> OperationResult<()> {
    state.update_settings(settings);
    OperationResult::success("Settings updated successfully", None)
}
