use crate::core::{
    AsyncTask, DatabaseInfo, DatabaseManager, DatabaseStatus, DatabaseType, GlobalSettings,
};
#[cfg(target_os = "macos")]
use crate::core::{get_homebrew_service_status, get_installed_databases_from_homebrew};
use crate::core::utils;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
struct AppStateData {
    databases: Vec<DatabaseInfo>,
    settings: GlobalSettings,
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub databases: Arc<Mutex<HashMap<String, DatabaseInfo>>>,
    pub settings: Arc<Mutex<GlobalSettings>>,
    pub db_manager: Arc<DatabaseManager>,
    pub tasks: Arc<Mutex<HashMap<String, AsyncTask>>>,
}

impl AppState {
    fn get_state_path() -> PathBuf {
        dirs::home_dir()
            .map(|p| p.join(".local-db").join("state.json"))
            .unwrap_or_else(|| PathBuf::from(".local-db/state.json"))
    }

    fn ensure_state_dir() -> Result<(), String> {
        let state_path = Self::get_state_path();
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create state directory: {}", e))?;
        }
        Ok(())
    }

    fn save_state(&self) -> Result<(), String> {
        Self::ensure_state_dir()?;

        let state_path = Self::get_state_path();
        let databases: Vec<DatabaseInfo> = {
            let db_map = self.databases.lock().unwrap();
            db_map.values().cloned().collect()
        };
        let settings = self.settings.lock().unwrap().clone();

        let state_data = AppStateData {
            databases,
            settings,
        };
        let json = serde_json::to_string_pretty(&state_data)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        fs::write(&state_path, json).map_err(|e| format!("Failed to write state file: {}", e))
    }

    fn load_state() -> Result<AppStateData, String> {
        let state_path = Self::get_state_path();

        if !state_path.exists() {
            return Ok(AppStateData {
                databases: Vec::new(),
                settings: GlobalSettings::default(),
            });
        }

        let content = fs::read_to_string(&state_path)
            .map_err(|e| format!("Failed to read state file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to deserialize state: {}", e))
    }

    pub fn new() -> Self {
        let state_data = Self::load_state().unwrap_or_else(|e| {
            eprintln!("Failed to load state: {}. Using defaults.", e);
            AppStateData {
                databases: Vec::new(),
                settings: GlobalSettings::default(),
            }
        });

        let settings = state_data.settings;
        let storage_path = PathBuf::from(&settings.default_storage_path);
        let db_manager = DatabaseManager::new(&storage_path);

        if let Err(e) = db_manager.init_directories() {
            eprintln!("Failed to initialize directories: {}", e);
        }

        let mut databases = HashMap::new();
        for mut db_info in state_data.databases {
            // 更新数据库的实际运行状态
            #[cfg(target_os = "macos")]
            {
                if let Some(service_name) = Self::get_brew_service_name(&db_info.db_type) {
                    if let Some(is_running) = get_homebrew_service_status(service_name) {
                        db_info.status = if is_running {
                            DatabaseStatus::Running
                        } else {
                            DatabaseStatus::Stopped
                        };
                        db_info.pid = None;
                        db_info.updated_at = utils::get_timestamp();
                    }
                }
            }
            databases.insert(db_info.id.clone(), db_info);
        }

        #[cfg(target_os = "macos")]
        {
            // 同步 Homebrew 已安装但不在状态中的数据库
            Self::sync_with_homebrew(&mut databases, &storage_path);
        }

        let app_state = Self {
            databases: Arc::new(Mutex::new(databases)),
            settings: Arc::new(Mutex::new(settings)),
            db_manager: Arc::new(db_manager),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        };

        // 保存同步后的状态
        let _ = app_state.save_state();

        app_state
    }

    /// 添加数据库
    ///
    /// 约束：同一种 `db_type` 只允许存在一个实例。
    pub fn add_database(&self, db_info: DatabaseInfo) -> Result<(), String> {
        let mut databases = self.databases.lock().unwrap();

        if databases.contains_key(&db_info.id) {
            return Err(format!("Database id already exists: {}", db_info.id));
        }

        let new_type: DatabaseType = db_info.db_type.clone();
        if let Some((existing_id, existing)) = databases
            .iter()
            .find(|(_, existing_db)| existing_db.db_type == new_type)
        {
            return Err(format!(
                "Database type already exists: {} (id={})",
                existing.db_type.as_str(),
                existing_id
            ));
        }

        databases.insert(db_info.id.clone(), db_info);
        drop(databases);

        self.save_state()
    }

    /// 获取数据库
    pub fn get_database(&self, id: &str) -> Option<DatabaseInfo> {
        let databases = self.databases.lock().unwrap();
        databases.get(id).cloned()
    }

    /// 获取所有数据库
    pub fn get_all_databases(&self) -> Vec<DatabaseInfo> {
        let databases = self.databases.lock().unwrap();
        databases.values().cloned().collect()
    }

    /// 更新数据库
    ///
    /// 为了保证"每种数据库只允许一个实例"，如果发现存在其它同 `db_type` 的条目，会被移除。
    pub fn update_database(&self, db_info: DatabaseInfo) {
        let mut databases = self.databases.lock().unwrap();

        let db_type = db_info.db_type.clone();
        let duplicate_ids: Vec<String> = databases
            .iter()
            .filter(|(id, existing)| *id != &db_info.id && existing.db_type == db_type)
            .map(|(id, _)| id.clone())
            .collect();

        for id in duplicate_ids {
            databases.remove(&id);
        }

        databases.insert(db_info.id.clone(), db_info);
        drop(databases);

        let _ = self.save_state();
    }

    /// 删除数据库
    pub fn remove_database(&self, id: &str) -> Option<DatabaseInfo> {
        let mut databases = self.databases.lock().unwrap();
        let result = databases.remove(id);
        drop(databases);

        let _ = self.save_state();
        result
    }

    /// 添加任务
    pub fn add_task(&self, task: AsyncTask) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task.id.clone(), task);
        drop(tasks);
    }

    /// 获取任务
    pub fn get_task(&self, id: &str) -> Option<AsyncTask> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(id).cloned()
    }

    /// 更新任务状态
    pub fn update_task<F>(&self, id: &str, updater: F)
    where
        F: FnOnce(&mut AsyncTask),
    {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(id) {
            updater(task);
            task.updated_at = utils::get_timestamp();
        }
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<AsyncTask> {
        let tasks = self.tasks.lock().unwrap();
        tasks.values().cloned().collect()
    }

    /// 移除任务
    pub fn remove_task(&self, id: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.remove(id);
    }

    /// 根据数据库类型获取数据库
    pub fn get_database_by_type(&self, db_type: &str) -> Option<DatabaseInfo> {
        let databases = self.databases.lock().unwrap();
        databases
            .values()
            .find(|db| db.db_type.as_str() == db_type)
            .cloned()
    }

    /// 获取设置
    pub fn get_settings(&self) -> GlobalSettings {
        let settings = self.settings.lock().unwrap();
        settings.clone()
    }

    /// 更新设置
    pub fn update_settings(&self, settings: GlobalSettings) {
        let mut current_settings = self.settings.lock().unwrap();
        *current_settings = settings;
        drop(current_settings);

        let _ = self.save_state();
    }

    /// 启动所有自动启动的数据库
    pub fn start_autostart_databases(&self) {
        let databases = self.get_all_databases();
        for mut db_info in databases {
            if db_info.auto_start {
                if let Err(e) = self.db_manager.start_database(&mut db_info) {
                    eprintln!("Failed to start database {}: {}", db_info.name, e);
                } else {
                    self.update_database(db_info);
                }
            }
        }
    }

    /// 获取数据库类型对应的 Homebrew 服务名称
    fn get_brew_service_name(db_type: &DatabaseType) -> Option<&'static str> {
        match db_type {
            DatabaseType::Redis => Some("redis"),
            DatabaseType::MySQL => Some("mysql"),
            DatabaseType::PostgreSQL => Some("postgresql"),
            DatabaseType::MongoDB => Some("mongodb-community@7.0"),
            DatabaseType::Qdrant => Some("qdrant"),
            DatabaseType::SeekDB => Some("seekdb"),
            DatabaseType::SurrealDB => Some("surreal"),
        }
    }

    /// 创建 Homebrew 数据库信息（用于同步已安装但未在状态中的数据库）
    #[cfg(target_os = "macos")]
    fn create_info_from_homebrew(
        db_type: DatabaseType,
        storage_path: &PathBuf,
        is_running: bool,
    ) -> DatabaseInfo {
        let formula = match &db_type {
            DatabaseType::Redis => "redis",
            DatabaseType::MySQL => "mysql",
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::MongoDB => "mongodb-community@7.0",
            DatabaseType::Qdrant => "qdrant",
            DatabaseType::SeekDB => "seekdb",
            DatabaseType::SurrealDB => "surreal",
        };

        // 获取安装路径（使用 brew --prefix）
        let install_path = std::process::Command::new("brew")
            .args(["--prefix", formula])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("/usr/local/opt/{}", formula));

        // 获取版本
        let version = std::process::Command::new("brew")
            .args(["list", "--versions", formula])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    text.split_whitespace()
                        .nth(1)
                        .map(|v| v.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "latest".to_string());

        let data_path = utils::get_db_data_path(storage_path, db_type.as_str());
        let log_path = utils::get_db_log_path(storage_path, db_type.as_str());
        let config_path = PathBuf::from(&install_path).join("etc").join(match &db_type {
            DatabaseType::Redis => "redis.conf",
            DatabaseType::MySQL => "my.cnf",
            DatabaseType::PostgreSQL => "postgresql.conf",
            DatabaseType::MongoDB => "mongod.conf",
            DatabaseType::Qdrant => "config.yaml",
            DatabaseType::SeekDB => "seekdb.conf",
            DatabaseType::SurrealDB => "surrealdb.yaml",
        });

        DatabaseInfo {
            id: format!("homebrew-sync-{}", db_type.as_str()),
            name: db_type.display_name().to_string(),
            db_type: db_type.clone(),
            version,
            install_path,
            data_path: data_path.to_string_lossy().to_string(),
            log_path: log_path.to_string_lossy().to_string(),
            port: db_type.default_port(),
            username: match &db_type {
                DatabaseType::MySQL => Some("root".to_string()),
                DatabaseType::PostgreSQL => Some("postgres".to_string()),
                _ => None,
            },
            password: None,
            config: Some(config_path.to_string_lossy().to_string()),
            status: if is_running {
                DatabaseStatus::Running
            } else {
                DatabaseStatus::Stopped
            },
            auto_start: false,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        }
    }

    /// 同步 Homebrew 已安装的数据库
    #[cfg(target_os = "macos")]
    fn sync_with_homebrew(databases: &mut HashMap<String, DatabaseInfo>, storage_path: &PathBuf) {
        let installed = get_installed_databases_from_homebrew();

        for db_type in installed {
            // 检查该类型的数据库是否已在状态中
            let type_str = db_type.as_str();
            let already_exists = databases.values().any(|db| db.db_type.as_str() == type_str);

            if !already_exists {
                let service_name = Self::get_brew_service_name(&db_type).unwrap_or("");
                let is_running =
                    get_homebrew_service_status(service_name).unwrap_or(false);

                let info = Self::create_info_from_homebrew(db_type, storage_path, is_running);
                eprintln!(
                    "Synced Homebrew installation: {} (status: {:?})",
                    info.name, info.status
                );
                databases.insert(info.id.clone(), info);
            }
        }

        // 移除已不再安装的数据库（可选）
        // 默认不移除，因为用户可能只是临时停止 Homebrew 服务
    }
}
