use crate::app::AppState;
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
    _app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // 检查是否已安装该类型的数据库
    if state.get_database_by_type(&params.db_type).is_some() {
        return Err("Database type already installed".to_string());
    }

    // 解析数据库类型
    let db_type: DatabaseType = match params.db_type.as_str() {
        "mysql" => DatabaseType::MySQL,
        "postgresql" => DatabaseType::PostgreSQL,
        "mongodb" => DatabaseType::MongoDB,
        "redis" => DatabaseType::Redis,
        "qdrant" => DatabaseType::Qdrant,
        "surrealdb" => DatabaseType::SurrealDB,
        _ => return Err("Unsupported database type".to_string()),
    };

    // 获取存储路径
    let settings = state.get_settings();
    let storage_path = std::path::PathBuf::from(&settings.default_storage_path);

    // 创建任务ID
    let task_id = format!(
        "install-{}-{}",
        params.db_type,
        crate::core::utils::generate_id()
    );

    // 创建任务
    let task = crate::core::AsyncTask {
        id: task_id.clone(),
        task_type: "install".to_string(),
        db_type: params.db_type.clone(),
        status: crate::core::TaskStatus::Pending,
        progress: 0,
        message: "Installation pending...".to_string(),
        error: None,
        created_at: crate::core::utils::get_timestamp(),
        updated_at: crate::core::utils::get_timestamp(),
    };
    state.add_task(task.clone());

    // 克隆需要在线程中使用的数据
    let db_type_clone = db_type.clone();
    let storage_path_clone = storage_path.clone();
    let version_param = params.version.clone();
    let port_param = params.port;
    let username_param = params.username.clone();
    let password_param = params.password.clone();

    // 克隆 Arc 指针以在线程中使用
    let databases_arc = state.databases.clone();
    let settings_arc = state.settings.clone();
    let tasks_arc = state.tasks.clone();
    let task_id_for_thread = task_id.clone();

    // 启动线程执行安装
    std::thread::spawn(move || {
        let task_id_clone = task_id_for_thread.clone();

        // 更新任务状态为运行中
        {
            let mut tasks = tasks_arc.lock().unwrap();
            if let Some(task) = tasks.get_mut(&task_id_clone) {
                task.status = crate::core::TaskStatus::Running;
                task.progress = 10;
                task.message = "Starting installation...".to_string();
                task.updated_at = crate::core::utils::get_timestamp();
            }
        }

        #[cfg(target_os = "macos")]
        let install_result: Result<crate::core::DatabaseInfo, String> = {
            let options = crate::core::macos::HomebrewInstallOptions {
                version: version_param.as_deref(),
                port: port_param,
                username: username_param.as_deref(),
                password: password_param.as_deref(),
                auto_start: true,
            };

            crate::core::macos::install_database_via_homebrew(
                &db_type_clone,
                &storage_path_clone,
                &options,
            )
            .map_err(|e| format!("Installation failed: {}", e))
        };

        #[cfg(not(target_os = "macos"))]
        let install_result: Result<crate::core::DatabaseInfo, String> = {
            // Create database info - simulate installation completion
            Ok(crate::core::DatabaseInfo {
                id: crate::core::utils::generate_id(),
                name: db_type_clone.display_name().to_string(),
                db_type: db_type_clone.clone(),
                version: version_param.unwrap_or_else(|| "latest".to_string()),
                install_path: storage_path_clone
                    .join("bin")
                    .join(db_type_clone.as_str())
                    .to_string_lossy()
                    .to_string(),
                data_path: crate::core::utils::get_db_data_path(
                    &storage_path_clone,
                    db_type_clone.as_str(),
                )
                .to_string_lossy()
                .to_string(),
                log_path: crate::core::utils::get_db_log_path(
                    &storage_path_clone,
                    db_type_clone.as_str(),
                )
                .to_string_lossy()
                .to_string(),
                port: port_param.unwrap_or_else(|| db_type_clone.default_port()),
                username: username_param,
                password: password_param,
                config: None,
                status: crate::core::DatabaseStatus::Stopped,
                auto_start: false,
                pid: None,
                created_at: crate::core::utils::get_timestamp(),
                updated_at: crate::core::utils::get_timestamp(),
            })
        };

        // 处理安装结果
        match install_result {
            Ok(db_info) => {
                // 添加数据库到状态
                let add_result = {
                    let mut databases = databases_arc.lock().unwrap();
                    if databases.contains_key(&db_info.id) {
                        Err(format!("Database id already exists: {}", db_info.id))
                    } else {
                        let new_type: crate::core::DatabaseType = db_info.db_type.clone();
                        if let Some((existing_id, _)) = databases
                            .iter()
                            .find(|(_, existing_db)| existing_db.db_type == new_type)
                        {
                            Err(format!(
                                "Database type already exists: {} (id={})",
                                new_type.as_str(),
                                existing_id
                            ))
                        } else {
                            databases.insert(db_info.id.clone(), db_info.clone());
                            // 保存状态
                            drop(databases);
                            save_app_state(&settings_arc, &databases_arc)
                        }
                    }
                };

                match add_result {
                    Ok(_) => {
                        let mut tasks = tasks_arc.lock().unwrap();
                        if let Some(task) = tasks.get_mut(&task_id_clone) {
                            task.status = crate::core::TaskStatus::Completed;
                            task.progress = 100;
                            task.message = "Installation completed successfully".to_string();
                            task.updated_at = crate::core::utils::get_timestamp();
                        }
                    }
                    Err(e) => {
                        let mut tasks = tasks_arc.lock().unwrap();
                        if let Some(task) = tasks.get_mut(&task_id_clone) {
                            task.status = crate::core::TaskStatus::Failed;
                            task.error = Some(format!("Failed to add database to state: {}", e));
                            task.message =
                                "Installation completed but failed to register".to_string();
                            task.updated_at = crate::core::utils::get_timestamp();
                        }
                    }
                }
            }
            Err(e) => {
                let mut tasks = tasks_arc.lock().unwrap();
                if let Some(task) = tasks.get_mut(&task_id_clone) {
                    task.status = crate::core::TaskStatus::Failed;
                    task.error = Some(e);
                    task.message = "Installation failed".to_string();
                    task.updated_at = crate::core::utils::get_timestamp();
                }
            }
        }
    });

    Ok(task_id)
}

/// 保存应用状态辅助函数
fn save_app_state(
    settings: &std::sync::Arc<std::sync::Mutex<crate::core::GlobalSettings>>,
    databases: &std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, crate::core::DatabaseInfo>>,
    >,
) -> Result<(), String> {
    fn get_state_path() -> std::path::PathBuf {
        dirs::home_dir()
            .map(|p| p.join(".local-db").join("state.json"))
            .unwrap_or_else(|| std::path::PathBuf::from(".local-db/state.json"))
    }

    fn ensure_state_dir() -> Result<(), String> {
        let state_path = get_state_path();
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create state directory: {}", e))?;
        }
        Ok(())
    }

    ensure_state_dir()?;

    let state_path = get_state_path();
    let databases_vec: Vec<crate::core::DatabaseInfo> = {
        let db_map = databases.lock().unwrap();
        db_map.values().cloned().collect()
    };
    let settings_clone = settings.lock().unwrap().clone();

    #[derive(serde::Serialize)]
    struct AppStateData {
        databases: Vec<crate::core::DatabaseInfo>,
        settings: crate::core::GlobalSettings,
    }

    let state_data = AppStateData {
        databases: databases_vec,
        settings: settings_clone,
    };
    let json = serde_json::to_string_pretty(&state_data)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    std::fs::write(&state_path, json).map_err(|e| format!("Failed to write state file: {}", e))
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

/// 获取任务状态
#[tauri::command]
pub fn get_task_status(state: State<AppState>, task_id: String) -> Option<crate::core::AsyncTask> {
    state.get_task(&task_id)
}

/// 同步所有数据库的运行状态
/// 此命令用于在前端页面加载完成后调用，检查并更新所有数据库的实际运行状态
/// 流程：
/// 1. 对于使用 PID 文件的数据库（Qdrant, SurrealDB），先检查 PID 文件
/// 2. 对于使用 Homebrew 的数据库（MySQL, PostgreSQL, MongoDB, Redis），检查 brew services list
/// 3. 更新数据库状态信息，纠正可能不正确的状态
#[tauri::command]
pub fn sync_databases_status(state: State<AppState>) -> Vec<DatabaseInfo> {
    #[cfg(target_os = "macos")]
    {
        use crate::core::macos::get_all_homebrew_services_status;
        use crate::core::DatabaseType;
        use std::path::Path;

        let databases = state.get_all_databases();
        
        // 一次性获取所有 Homebrew 服务的状态，提高效率
        let brew_services = get_all_homebrew_services_status();

        for mut db_info in databases {
            let actual_status = match db_info.db_type {
                // Qdrant 通过 PID 文件检查状态
                DatabaseType::Qdrant => {
                    let pid_path = Path::new(&db_info.data_path).join("qdrant.pid");
                    check_pid_file_status(&pid_path)
                }
                // SurrealDB 通过 PID 文件检查状态
                DatabaseType::SurrealDB => {
                    let pid_path = Path::new(&db_info.data_path).join("surrealdb.pid");
                    check_pid_file_status(&pid_path)
                }
                // 其他数据库通过 brew services 检查状态
                DatabaseType::Redis => {
                    get_database_status_from_brew(&brew_services, "redis")
                        .unwrap_or(DatabaseStatus::Stopped)
                }
                DatabaseType::MySQL => {
                    // 尝试多个常见的 MySQL 版本标签
                    get_database_status_from_brew(&brew_services, "mysql@8.4")
                        .or_else(|| get_database_status_from_brew(&brew_services, "mysql@8.0"))
                        .or_else(|| get_database_status_from_brew(&brew_services, "mysql"))
                        .unwrap_or(DatabaseStatus::Stopped)
                }
                DatabaseType::PostgreSQL => {
                    // 尝试多个常见的 PostgreSQL 版本标签
                    get_database_status_from_brew(&brew_services, "postgresql@18")
                        .or_else(|| get_database_status_from_brew(&brew_services, "postgresql@17"))
                        .or_else(|| get_database_status_from_brew(&brew_services, "postgresql@16"))
                        .or_else(|| get_database_status_from_brew(&brew_services, "postgresql"))
                        .unwrap_or(DatabaseStatus::Stopped)
                }
                DatabaseType::MongoDB => {
                    // 尝试多个常见的 MongoDB 版本标签
                    get_database_status_from_brew(&brew_services, "mongodb-community@7.0")
                        .or_else(|| get_database_status_from_brew(&brew_services, "mongodb-community@6.0"))
                        .or_else(|| get_database_status_from_brew(&brew_services, "mongodb-community"))
                        .unwrap_or(DatabaseStatus::Stopped)
                }
            };

            // 如果状态有变化，更新数据库信息
            if db_info.status != actual_status {
                db_info.status = actual_status.clone();
                db_info.updated_at = crate::core::utils::get_timestamp();
                state.update_database(db_info);
            }
        }
    }

    // 返回更新后的数据库列表
    state.get_all_databases()
}

/// 从 Homebrew 服务状态映射中获取数据库状态
#[cfg(target_os = "macos")]
fn get_database_status_from_brew(
    brew_services: &std::collections::HashMap<String, bool>,
    service_name: &str,
) -> Option<DatabaseStatus> {
    brew_services.get(service_name).map(|&running| {
        if running {
            DatabaseStatus::Running
        } else {
            DatabaseStatus::Stopped
        }
    })
}

/// 检查 PID 文件中的进程是否存在
#[cfg(target_os = "macos")]
fn check_pid_file_status(pid_path: &std::path::Path) -> DatabaseStatus {
    use std::fs;

    if !pid_path.exists() {
        return DatabaseStatus::Stopped;
    }

    if let Ok(pid_str) = fs::read_to_string(pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if kill(Pid::from_raw(pid), Signal::SIGCONT).is_ok() {
                    return DatabaseStatus::Running;
                }
            }
        }
    }

    DatabaseStatus::Stopped
}
