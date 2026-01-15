use crate::core::{DatabaseInfo, DatabaseType};
use anyhow::Result;
use std::path::Path;

/// Options used when installing a database on Windows.
pub struct WindowsInstallOptions<'a> {
    pub version: Option<&'a str>,
    pub port: Option<u16>,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub auto_start: bool,
}

impl<'a> Default for WindowsInstallOptions<'a> {
    fn default() -> Self {
        Self {
            version: None,
            port: None,
            username: None,
            password: None,
            auto_start: true,
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use crate::core::{utils, DatabaseStatus};
    use anyhow::{bail, Context};
    use reqwest::blocking::get;
    use std::fs;
    use std::process::Command;

    /// 安装数据库到 Windows
    pub fn install_database(
        db_type: &DatabaseType,
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        match db_type {
            DatabaseType::Redis => install_redis(storage_path, options),
            DatabaseType::MongoDB => install_mongodb(storage_path, options),
            DatabaseType::Qdrant => install_qdrant(storage_path, options),
            DatabaseType::SurrealDB => install_surrealdb(storage_path, options),
            DatabaseType::MySQL => install_mysql(storage_path, options),
            DatabaseType::PostgreSQL => install_postgresql(storage_path, options),
            DatabaseType::Neo4j => install_neo4j(storage_path, options),
            DatabaseType::SeekDB => install_seekdb(storage_path, options),
        }
    }

    /// 启动数据库服务
    pub fn start_service(db_info: &DatabaseInfo) -> Result<()> {
        match db_info.db_type {
            DatabaseType::Qdrant => start_qdrant_process(db_info),
            DatabaseType::SurrealDB => start_surrealdb_process(db_info),
            DatabaseType::Redis => start_redis_process(db_info),
            DatabaseType::MongoDB => start_mongodb_process(db_info),
            DatabaseType::MySQL => start_mysql_process(db_info),
            DatabaseType::PostgreSQL => start_postgresql_process(db_info),
            DatabaseType::Neo4j => start_neo4j_process(db_info),
            DatabaseType::SeekDB => start_seekdb_process(db_info),
        }
    }

    /// 停止数据库服务
    pub fn stop_service(db_info: &DatabaseInfo) -> Result<()> {
        let pid_file = match db_info.db_type {
            DatabaseType::Qdrant => "qdrant.pid",
            DatabaseType::SurrealDB => "surrealdb.pid",
            DatabaseType::Redis => "redis.pid",
            DatabaseType::MongoDB => "mongodb.pid",
            DatabaseType::MySQL => "mysql.pid",
            DatabaseType::PostgreSQL => "postgresql.pid",
            DatabaseType::Neo4j => "neo4j.pid",
            DatabaseType::SeekDB => "seekdb.pid",
        };

        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join(pid_file);

        if pid_path.exists() {
            let pid_str = fs::read_to_string(&pid_path)?;
            let pid = pid_str.trim();

            // Windows: 使用 taskkill 强制停止
            Command::new("taskkill")
                .arg("/PID")
                .arg(pid)
                .arg("/F")
                .output()
                .context("Failed to stop process via taskkill")?;

            let _ = fs::remove_file(&pid_path);
        }

        Ok(())
    }

    // --- Redis Implementation ---

    fn install_redis(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "redis");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "redis");
        let logs_dir = utils::get_db_log_path(storage_path, "redis");
        let config_dir = utils::get_db_config_path(storage_path, "redis");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        utils::ensure_dir(&config_dir)?;

        let binary_path = bin_dir.join("redis-server.exe");
        let config_path = config_dir.join("redis.conf");
        let log_file = logs_dir.join("redis.log");

        if !binary_path.exists() {
            // Redis Windows 移植版下载地址 (tporadowski/redis)
            let url = "https://github.com/tporadowski/redis/releases/download/v5.0.14.1/Redis-x64-5.0.14.1.zip";
            download_and_extract_zip(url, &bin_dir)?;
        }

        let port = options.port.unwrap_or(6379);
        let config_content = format!(
            "port {port}\nbind 127.0.0.1\ndir \"{data_path}\"\nlogfile \"{log_path}\"\n",
            port = port,
            data_path = data_dir.to_string_lossy().replace("\\", "/"),
            log_path = log_file.to_string_lossy().replace("\\", "/")
        );
        fs::write(&config_path, config_content)?;

        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "Redis".to_string(),
            db_type: DatabaseType::Redis,
            version: "5.0.14.1".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: None,
            password: options.password.map(|s| s.to_string()),
            config: Some(config_path.to_string_lossy().to_string()),
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_redis_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_redis_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path).join("redis-server.exe");
        let config_path = db_info.config.as_ref().context("Config path missing")?;
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("redis.pid");

        let child = Command::new(binary_path)
            .arg(config_path)
            .spawn()
            .context("Failed to start Redis")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- Qdrant Implementation ---

    fn install_qdrant(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "qdrant");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "qdrant");
        let logs_dir = utils::get_db_log_path(storage_path, "qdrant");
        let config_dir = utils::get_db_config_path(storage_path, "qdrant");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        utils::ensure_dir(&config_dir)?;

        let binary_path = bin_dir.join("qdrant.exe");
        let config_path = config_dir.join("config.yaml");
        let log_file = logs_dir.join("qdrant.log");

        if !binary_path.exists() {
            let arch = if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x86_64"
            };
            let url = format!(
                "https://github.com/qdrant/qdrant/releases/latest/download/qdrant-{}-pc-windows-msvc.zip",
                arch
            );
            download_and_extract_zip(&url, &bin_dir)?;
        }

        let port = options.port.unwrap_or(6333);
        let config_content = format!(
            "service:\n  http_port: {port}\nstorage:\n  storage_path: {data_path}\n",
            port = port,
            data_path = data_dir.to_string_lossy().replace("\\", "/")
        );
        fs::write(&config_path, config_content)?;

        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "Qdrant".to_string(),
            db_type: DatabaseType::Qdrant,
            version: "latest".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: None,
            password: options.password.map(|s| s.to_string()),
            config: Some(config_path.to_string_lossy().to_string()),
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_qdrant_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_qdrant_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path).join("qdrant.exe");
        let config_path = db_info.config.as_ref().context("Config path missing")?;
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("qdrant.pid");

        let child = Command::new(binary_path)
            .arg("--config-path")
            .arg(config_path)
            .spawn()
            .context("Failed to start Qdrant")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- SurrealDB Implementation ---

    fn install_surrealdb(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "surrealdb");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "surrealdb");
        let logs_dir = utils::get_db_log_path(storage_path, "surrealdb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("surreal.exe");
        let log_file = logs_dir.join("surreal.log");

        if !binary_path.exists() {
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "amd64"
            };
            let url = format!(
                "https://github.com/surrealdb/surrealdb/releases/latest/download/surreal-v2.1.4-windows-{}.zip",
                arch
            );
            download_and_extract_zip(&url, &bin_dir)?;
        }

        let port = options.port.unwrap_or(8000);
        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "SurrealDB".to_string(),
            db_type: DatabaseType::SurrealDB,
            version: "latest".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: Some(options.username.unwrap_or("root").to_string()),
            password: Some(options.password.unwrap_or("root").to_string()),
            config: None,
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_surrealdb_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_surrealdb_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path).join("surreal.exe");
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("surrealdb.pid");

        let child = Command::new(binary_path)
            .arg("start")
            .arg("--bind")
            .arg(format!("127.0.0.1:{}", db_info.port))
            .arg("--user")
            .arg(db_info.username.as_deref().unwrap_or("root"))
            .arg("--pass")
            .arg(db_info.password.as_deref().unwrap_or("root"))
            .arg(format!("rocksdb://{}", data_dir.to_string_lossy()))
            .spawn()
            .context("Failed to start SurrealDB")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- MongoDB Implementation ---

    fn install_mongodb(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "mongodb");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "mongodb");
        let logs_dir = utils::get_db_log_path(storage_path, "mongodb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("bin").join("mongod.exe");
        let log_file = logs_dir.join("mongodb.log");

        if !binary_path.exists() {
            // MongoDB 7.0 社区版下载地址
            let url = "https://fastdl.mongodb.org/windows/mongodb-windows-x86_64-7.0.9.zip";
            download_and_extract_zip(url, &bin_dir)?;
        }

        let port = options.port.unwrap_or(27017);
        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "MongoDB".to_string(),
            db_type: DatabaseType::MongoDB,
            version: "7.0.9".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: None,
            password: None,
            config: None,
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_mongodb_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_mongodb_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path)
            .join("bin")
            .join("mongod.exe");
        let data_dir = Path::new(&db_info.data_path);
        let log_file = Path::new(&db_info.log_path);
        let pid_path = data_dir.join("mongodb.pid");

        let child = Command::new(binary_path)
            .arg("--dbpath")
            .arg(data_dir)
            .arg("--port")
            .arg(db_info.port.to_string())
            .arg("--logpath")
            .arg(log_file)
            .spawn()
            .context("Failed to start MongoDB")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- MySQL Implementation ---

    fn install_mysql(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "mysql");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "mysql");
        let logs_dir = utils::get_db_log_path(storage_path, "mysql");
        let config_dir = utils::get_db_config_path(storage_path, "mysql");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        utils::ensure_dir(&config_dir)?;

        let binary_path = bin_dir.join("bin").join("mysqld.exe");
        let log_file = logs_dir.join("mysql.log");

        if !binary_path.exists() {
            // MySQL 8.4 社区版下载地址
            let url = "https://dev.mysql.com/get/Downloads/MySQL-8.4/mysql-8.4.0-winx64.zip";
            download_and_extract_zip(url, &bin_dir)?;
        }

        // 初始化 MySQL 数据目录 (无密码)
        if fs::read_dir(&data_dir)?.count() == 0 {
            Command::new(&binary_path)
                .arg("--initialize-insecure")
                .arg(format!("--datadir={}", data_dir.to_string_lossy()))
                .output()
                .context("Failed to initialize MySQL")?;
        }

        let port = options.port.unwrap_or(3306);
        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "MySQL".to_string(),
            db_type: DatabaseType::MySQL,
            version: "8.4.0".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: options.username.map(|s| s.to_string()).or(Some("root".to_string())),
            password: options.password.map(|s| s.to_string()),
            config: None,
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_mysql_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_mysql_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path)
            .join("bin")
            .join("mysqld.exe");
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("mysql.pid");

        let child = Command::new(binary_path)
            .arg(format!("--port={}", db_info.port))
            .arg(format!("--datadir={}", data_dir.to_string_lossy()))
            .arg("--console")
            .spawn()
            .context("Failed to start MySQL")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- PostgreSQL Implementation ---

    fn install_postgresql(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "postgresql");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "postgresql");
        let logs_dir = utils::get_db_log_path(storage_path, "postgresql");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("bin").join("pg_ctl.exe");
        let initdb_path = bin_dir.join("bin").join("initdb.exe");
        let log_file = logs_dir.join("postgresql.log");

        if !binary_path.exists() {
            // PostgreSQL 18 社区版下载地址 (EDB 提供的 ZIP 版)
            let url = "https://get.enterprisedb.com/postgresql/postgresql-18.2-1-windows-x64-binaries.zip";
            download_and_extract_zip(url, &bin_dir)?;
        }

        // 初始化 PostgreSQL 数据目录
        if fs::read_dir(&data_dir)?.count() == 0 {
            Command::new(initdb_path)
                .arg("-D")
                .arg(&data_dir)
                .arg("-U")
                .arg(options.username.unwrap_or("postgres"))
                .arg("--auth=trust")
                .output()
                .context("Failed to initialize PostgreSQL")?;
        }

        let port = options.port.unwrap_or(5432);
        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "PostgreSQL".to_string(),
            db_type: DatabaseType::PostgreSQL,
            version: "18.2".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: options.username.map(|s| s.to_string()).or(Some("postgres".to_string())),
            password: options.password.map(|s| s.to_string()),
            config: None,
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_postgresql_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_postgresql_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path)
            .join("bin")
            .join("pg_ctl.exe");
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("postgresql.pid");

        let child = Command::new(binary_path)
            .arg("start")
            .arg("-D")
            .arg(data_dir)
            .arg("-o")
            .arg(format!("-p {}", db_info.port))
            .spawn()
            .context("Failed to start PostgreSQL")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- Neo4j Implementation ---

    fn install_neo4j(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "neo4j");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "neo4j");
        let logs_dir = utils::get_db_log_path(storage_path, "neo4j");
        let config_dir = utils::get_db_config_path(storage_path, "neo4j");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        utils::ensure_dir(&config_dir)?;

        let binary_path = bin_dir.join("bin").join("neo4j.bat");
        let log_file = logs_dir.join("neo4j.log");

        if !binary_path.exists() {
            let url = "https://neo4j.com/artifact.php?name=neo4j-community-5.20.0-windows.zip";
            download_and_extract_zip(url, &bin_dir)?;
        }

        let port = options.port.unwrap_or(7474);
        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "Neo4j".to_string(),
            db_type: DatabaseType::Neo4j,
            version: "5.20.0".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: options.username.map(|s| s.to_string()).or(Some("neo4j".to_string())),
            password: options.password.map(|s| s.to_string()).or(Some("password".to_string())),
            config: None,
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_neo4j_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_neo4j_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path)
            .join("bin")
            .join("neo4j.bat");
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("neo4j.pid");

        let child = Command::new("cmd")
            .arg("/C")
            .arg(binary_path)
            .arg("console")
            .spawn()
            .context("Failed to start Neo4j")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- SeekDB Implementation ---

    fn install_seekdb(
        storage_path: &Path,
        options: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "seekdb");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "seekdb");
        let logs_dir = utils::get_db_log_path(storage_path, "seekdb");
        let config_dir = utils::get_db_config_path(storage_path, "seekdb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        utils::ensure_dir(&config_dir)?;

        let binary_path = bin_dir.join("seekdb.exe");
        let config_path = config_dir.join("seekdb.conf");
        let log_file = logs_dir.join("seekdb.log");

        if !binary_path.exists() {
            let url = "https://github.com/seekdb/seekdb/releases/download/v0.1.0/seekdb_0.1.0_windows_amd64.zip";
            download_and_extract_zip(url, &bin_dir)?;
        }

        let port = options.port.unwrap_or(8080);
        let config_content = format!(
            "[server]\nport = {port}\nhost = 127.0.0.1\n\n[data]\ndir = \"{data_path}\"\n\n[logging]\nlevel = info\nfile = \"{log_path}\"\n",
            port = port,
            data_path = data_dir.to_string_lossy().replace("\\", "/"),
            log_path = log_file.to_string_lossy().replace("\\", "/")
        );
        fs::write(&config_path, config_content)?;

        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "SeekDB".to_string(),
            db_type: DatabaseType::SeekDB,
            version: "0.1.0".to_string(),
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: options.username.map(|s| s.to_string()),
            password: options.password.map(|s| s.to_string()),
            config: Some(config_path.to_string_lossy().to_string()),
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        if options.auto_start {
            start_seekdb_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_seekdb_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path).join("seekdb.exe");
        let config_path = db_info.config.as_ref().context("Config path missing")?;
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("seekdb.pid");

        let child = Command::new(binary_path)
            .arg("server")
            .arg("--config")
            .arg(config_path)
            .spawn()
            .context("Failed to start SeekDB")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- Helper Functions ---

    fn download_and_extract_zip(url: &str, target_dir: &Path) -> Result<()> {
        let response = get(url).context("Failed to download file")?;
        let bytes = response.bytes().context("Failed to get bytes")?;
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).context("Failed to open zip archive")?;

        // 检查是否所有文件都在同一个根目录下
        let mut common_prefix = None;
        let mut is_first = true;

        for i in 0..archive.len() {
            let file = archive.by_index(i).context("Failed to get file from zip")?;
            let name = file.name();
            if name.starts_with("__MACOSX") || name.contains(".DS_Store") {
                continue;
            }

            let path = Path::new(name);
            let first_comp = path
                .components()
                .next()
                .and_then(|c| c.as_os_str().to_str());

            if is_first {
                common_prefix = first_comp.map(|s| s.to_string());
                is_first = false;
            } else if common_prefix.as_deref() != first_comp {
                common_prefix = None;
                break;
            }
        }

        // 如果 ZIP 只有一个根目录且是个目录，我们就剥离它
        let should_strip = if let Some(ref prefix) = common_prefix {
            // 确保这个前缀确实是一个目录（以 / 结尾或 ZIP 中有对应的目录条目）
            archive.by_name(&format!("{}/", prefix)).is_ok()
                || archive.by_name(prefix).map(|f| f.is_dir()).unwrap_or(false)
        } else {
            false
        };

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).context("Failed to get file from zip")?;
            let name = file.name().to_string();

            if name.starts_with("__MACOSX") || name.contains(".DS_Store") {
                continue;
            }

            let relative_path = if should_strip {
                let prefix = common_prefix.as_ref().unwrap();
                if name == *prefix || name == format!("{}/", prefix) {
                    continue; // 跳过根目录条目本身
                }
                name.strip_prefix(prefix)
                    .and_then(|s| s.strip_prefix('/'))
                    .unwrap_or(&name)
                    .to_string()
            } else {
                name
            };

            if relative_path.is_empty() {
                continue;
            }

            let outpath = target_dir.join(relative_path);

            if file.is_dir() {
                fs::create_dir_all(&outpath).context("Failed to create directory")?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).context("Failed to create directory")?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).context("Failed to create file")?;
                std::io::copy(&mut file, &mut outfile).context("Failed to copy file")?;
            }
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;
    use anyhow::bail;
    pub fn install_database(
        _: &DatabaseType,
        _: &Path,
        _: &WindowsInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        bail!("Not supported on this OS")
    }
    pub fn start_service(_: &DatabaseInfo) -> Result<()> {
        bail!("Not supported on this OS")
    }
    pub fn stop_service(_: &DatabaseInfo) -> Result<()> {
        bail!("Not supported on this OS")
    }
}

pub use imp::*;
