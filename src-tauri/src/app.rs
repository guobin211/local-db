use crate::core::{
    AsyncTask, DatabaseInfo, DatabaseManager, GlobalSettings,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri_plugin_log::log;

#[derive(Debug, Serialize, Deserialize)]
struct AppStateData {
    databases: Vec<DatabaseInfo>,
    settings: GlobalSettings,
}

/// 应用状态
#[derive(Clone, Debug)]
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

        // 从保存的状态恢复数据库列表
        let databases: HashMap<String, DatabaseInfo> = state_data
            .databases
            .into_iter()
            .map(|db| (db.id.clone(), db))
            .collect();

        let app_state = Self {
            databases: Arc::new(Mutex::new(databases)),
            settings: Arc::new(Mutex::new(settings)),
            db_manager: Arc::new(db_manager),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        };
        log::info!("App state initialized. {:?}", app_state);
        app_state
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
}
