mod app;
mod command;
mod core;

use app::AppState;
use tauri::Manager;
use tauri_plugin_log::log;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 创建应用状态
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            app.get_webview_window("main")
                .map(|window| {
                    window.show().unwrap_or_else(|_| log::info!("window show"));
                })
                .unwrap_or_else(|| {
                    log::error!("Failed to get main window");
                });
        }))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(app_state.clone())
        .setup(move |_app| {
            // 启动自动启动的数据库
            app_state.start_autostart_databases();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 数据库命令
            command::get_databases,
            command::get_database,
            command::get_database_by_type,
            command::start_database,
            command::stop_database,
            command::restart_database,
            command::get_database_status,
            command::delete_database,
            command::install_database,
            command::update_database_autostart,
            // 设置命令
            command::get_settings,
            command::update_settings,
            // 系统信息命令
            command::get_system_info,
            command::get_cpu_usage,
            command::get_memory_info,
            command::get_disk_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
