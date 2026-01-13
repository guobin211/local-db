use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 数据库类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    MongoDB,
    Redis,
    Qdrant,
    SurrealDB,
}

impl DatabaseType {
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseType::MySQL => "mysql",
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::MongoDB => "mongodb",
            DatabaseType::Redis => "redis",
            DatabaseType::Qdrant => "qdrant",
            DatabaseType::SurrealDB => "surrealdb",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            DatabaseType::MySQL => "MySQL",
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::MongoDB => "MongoDB",
            DatabaseType::Redis => "Redis",
            DatabaseType::Qdrant => "Qdrant",
            DatabaseType::SurrealDB => "SurrealDB",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::MySQL => 3306,
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::MongoDB => 27017,
            DatabaseType::Redis => 6379,
            DatabaseType::Qdrant => 6333,
            DatabaseType::SurrealDB => 8000,
        }
    }
}

/// 数据库状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseStatus {
    Running,
    Stopped,
    NotInstalled,
}

/// 数据库信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub db_type: DatabaseType,
    pub version: String,
    pub install_path: String,
    pub data_path: String,
    pub log_path: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub config: Option<String>,
    pub status: DatabaseStatus,
    pub auto_start: bool,
    pub pid: Option<u32>,
    pub created_at: String,
    pub updated_at: String,
}

/// 数据库配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DatabaseConfig {
    pub port: Option<u16>,
    pub data_path: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub config: Option<HashMap<String, String>>,
}

/// 备份记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BackupRecord {
    pub id: String,
    pub database_id: String,
    pub file_path: String,
    pub file_size: u64,
    pub created_at: String,
}

/// 全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub default_storage_path: String,
    pub auto_start: bool,
    pub theme: String,
    pub language: String,
    pub auto_backup: bool,
    pub backup_frequency: String,
    pub backup_retention_days: u32,
    pub log_level: String,
    pub log_retention_days: u32,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            default_storage_path: Self::get_default_storage_path(),
            auto_start: false,
            theme: "light".to_string(),
            language: "en".to_string(),
            auto_backup: false,
            backup_frequency: "daily".to_string(),
            backup_retention_days: 7,
            log_level: "info".to_string(),
            log_retention_days: 7,
        }
    }
}

impl GlobalSettings {
    fn get_default_storage_path() -> String {
        dirs::home_dir()
            .map(|p| p.join(".local-db").to_string_lossy().to_string())
            .unwrap_or_else(|| ".local-db".to_string())
    }
}

/// 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> OperationResult<T> {
    pub fn success(message: impl Into<String>, data: Option<T>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
}

/// 任务状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// 异步任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTask {
    pub id: String,
    pub task_type: String,
    pub db_type: String,
    pub status: TaskStatus,
    pub progress: u8,
    pub message: String,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
