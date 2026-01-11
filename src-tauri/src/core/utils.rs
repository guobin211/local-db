use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// 确保目录存在
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 获取默认存储路径
#[allow(dead_code)]
pub fn get_default_storage_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".local-db"))
        .unwrap_or_else(|| PathBuf::from(".local-db"))
}

/// 获取数据库的bin目录
pub fn get_db_bin_path(storage_path: &Path, db_name: &str) -> PathBuf {
    storage_path.join("bin").join(db_name)
}

/// 获取数据库的config目录
#[allow(dead_code)]
pub fn get_db_config_path(storage_path: &Path, db_name: &str) -> PathBuf {
    storage_path.join("config").join(db_name)
}

/// 获取数据库的data目录
pub fn get_db_data_path(storage_path: &Path, db_name: &str) -> PathBuf {
    storage_path.join("data").join(db_name)
}

/// 获取数据库的log目录
pub fn get_db_log_path(storage_path: &Path, db_name: &str) -> PathBuf {
    storage_path.join("logs").join(db_name)
}

/// 获取数据库的backup目录
#[allow(dead_code)]
pub fn get_db_backup_path(storage_path: &Path, db_name: &str) -> PathBuf {
    storage_path.join("backups").join(db_name)
}

/// 生成唯一ID
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{}", timestamp)
}

/// 获取当前时间戳字符串
pub fn get_timestamp() -> String {
    chrono::Local::now().to_rfc3339()
}

/// 格式化文件大小
#[allow(dead_code)]
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
