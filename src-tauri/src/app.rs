use crate::core::{DatabaseInfo, DatabaseManager, DatabaseType, GlobalSettings};
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
        for db_info in state_data.databases {
            databases.insert(db_info.id.clone(), db_info);
        }

        Self {
            databases: Arc::new(Mutex::new(databases)),
            settings: Arc::new(Mutex::new(settings)),
            db_manager: Arc::new(db_manager),
        }
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
}
