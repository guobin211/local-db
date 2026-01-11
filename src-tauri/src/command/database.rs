use crate::app::AppState;
#[cfg(target_os = "macos")]
use crate::core::macos::{install_database_via_homebrew, HomebrewInstallOptions};
use crate::core::{DatabaseInfo, DatabaseStatus, DatabaseType, OperationResult};
use tauri::State;

/// 获取所有数据库列表
#[tauri::command]
pub fn get_databases(state: State<AppState>) -> Vec<DatabaseInfo> {
    state.get_all_databases()
}

/// 根据ID获取数据库信息
#[tauri::command]
pub fn get_database(state: State<AppState>, id: String) -> Option<DatabaseInfo> {
    state.get_database(&id)
}

/// 根据数据库类型获取数据库
#[tauri::command]
pub fn get_database_by_type(state: State<AppState>, db_type: String) -> Option<DatabaseInfo> {
    state.get_database_by_type(&db_type)
}

/// 启动数据库
#[tauri::command]
pub fn start_database(state: State<AppState>, id: String) -> OperationResult<()> {
    match state.get_database(&id) {
        Some(mut db_info) => match state.db_manager.start_database(&mut db_info) {
            Ok(result) => {
                if result.success {
                    state.update_database(db_info);
                }
                result
            }
            Err(e) => OperationResult::error(format!("Failed to start database: {}", e)),
        },
        None => OperationResult::error("Database not found"),
    }
}

/// 停止数据库
#[tauri::command]
pub fn stop_database(state: State<AppState>, id: String) -> OperationResult<()> {
    match state.get_database(&id) {
        Some(mut db_info) => match state.db_manager.stop_database(&mut db_info) {
            Ok(result) => {
                if result.success {
                    state.update_database(db_info);
                }
                result
            }
            Err(e) => OperationResult::error(format!("Failed to stop database: {}", e)),
        },
        None => OperationResult::error("Database not found"),
    }
}

/// 重启数据库
#[tauri::command]
pub fn restart_database(state: State<AppState>, id: String) -> OperationResult<()> {
    match state.get_database(&id) {
        Some(mut db_info) => {
            // 先停止
            if let Err(e) = state.db_manager.stop_database(&mut db_info) {
                return OperationResult::error(format!("Failed to stop database: {}", e));
            }

            // 等待一会儿
            std::thread::sleep(std::time::Duration::from_secs(2));

            // 再启动
            match state.db_manager.start_database(&mut db_info) {
                Ok(result) => {
                    if result.success {
                        state.update_database(db_info);
                    }
                    result
                }
                Err(e) => OperationResult::error(format!("Failed to start database: {}", e)),
            }
        }
        None => OperationResult::error("Database not found"),
    }
}

/// 获取数据库状态
#[tauri::command]
pub fn get_database_status(state: State<AppState>, id: String) -> Option<DatabaseStatus> {
    state
        .get_database(&id)
        .map(|db_info| state.db_manager.get_status(&db_info))
}

/// 删除数据库
#[tauri::command]
pub fn delete_database(state: State<AppState>, id: String, with_data: bool) -> OperationResult<()> {
    match state.get_database(&id) {
        Some(mut db_info) => {
            // 如果数据库正在运行，先停止
            if state.db_manager.get_status(&db_info) == DatabaseStatus::Running {
                if let Err(e) = state.db_manager.stop_database(&mut db_info) {
                    return OperationResult::error(format!("Failed to stop database: {}", e));
                }
            }

            // 如果需要删除数据
            if with_data {
                if let Err(e) = std::fs::remove_dir_all(&db_info.data_path) {
                    return OperationResult::error(format!("Failed to delete data: {}", e));
                }
            }

            // 从状态中移除
            state.remove_database(&id);

            OperationResult::success("Database deleted successfully", None)
        }
        None => OperationResult::error("Database not found"),
    }
}

/// 安装数据库参数
#[derive(serde::Deserialize)]
pub struct InstallDatabaseParams {
    pub db_type: String,
    pub version: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// 安装数据库
#[tauri::command]
pub fn install_database(
    state: State<AppState>,
    params: InstallDatabaseParams,
) -> OperationResult<DatabaseInfo> {
    // 检查是否已安装该类型的数据库
    if state.get_database_by_type(&params.db_type).is_some() {
        return OperationResult::error("Database type already installed");
    }

    // 解析数据库类型
    let db_type: DatabaseType = match params.db_type.as_str() {
        "mysql" => DatabaseType::MySQL,
        "postgresql" => DatabaseType::PostgreSQL,
        "mongodb" => DatabaseType::MongoDB,
        "redis" => DatabaseType::Redis,
        "qdrant" => DatabaseType::Qdrant,
        "neo4j" => DatabaseType::Neo4j,
        "seekdb" => DatabaseType::SeekDB,
        "surrealdb" => DatabaseType::SurrealDB,
        _ => return OperationResult::error("Unsupported database type"),
    };

    // 获取存储路径
    let settings = state.get_settings();
    let storage_path = std::path::PathBuf::from(&settings.default_storage_path);

    #[cfg(target_os = "macos")]
    {
        let options = HomebrewInstallOptions {
            version: params.version.as_deref(),
            port: params.port,
            username: params.username.as_deref(),
            password: params.password.as_deref(),
            auto_start: true,
        };

        match install_database_via_homebrew(&db_type, &storage_path, &options) {
            Ok(db_info) => {
                if let Err(message) = state.add_database(db_info.clone()) {
                    return OperationResult::error(message);
                }
                OperationResult::success("Database installed successfully", Some(db_info))
            }
            Err(err) => OperationResult::error(format!("Failed to install via Homebrew: {}", err)),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // 创建数据库信息 - 模拟安装完成后的状态
        let db_info = DatabaseInfo {
            id: crate::core::utils::generate_id(),
            name: db_type.display_name().to_string(),
            db_type: db_type.clone(),
            version: params.version.unwrap_or_else(|| "latest".to_string()),
            install_path: storage_path
                .join("bin")
                .join(db_type.as_str())
                .to_string_lossy()
                .to_string(),
            data_path: crate::core::utils::get_db_data_path(&storage_path, db_type.as_str())
                .to_string_lossy()
                .to_string(),
            log_path: crate::core::utils::get_db_log_path(&storage_path, db_type.as_str())
                .to_string_lossy()
                .to_string(),
            port: params.port.unwrap_or_else(|| db_type.default_port()),
            username: params.username,
            password: params.password,
            config: None,
            status: DatabaseStatus::Stopped, // 安装完成后状态为 Stopped
            auto_start: false,
            pid: None,
            created_at: crate::core::utils::get_timestamp(),
            updated_at: crate::core::utils::get_timestamp(),
        };

        if let Err(message) = state.add_database(db_info.clone()) {
            return OperationResult::error(message);
        }

        OperationResult::success("Database installed successfully", Some(db_info))
    }
}

/// 更新数据库自启动设置
#[tauri::command]
pub fn update_database_autostart(
    state: State<AppState>,
    id: String,
    auto_start: bool,
) -> OperationResult<()> {
    match state.get_database(&id) {
        Some(mut db_info) => {
            db_info.auto_start = auto_start;
            db_info.updated_at = crate::core::utils::get_timestamp();
            state.update_database(db_info);
            OperationResult::success("Autostart setting updated", None)
        }
        None => OperationResult::error("Database not found"),
    }
}
