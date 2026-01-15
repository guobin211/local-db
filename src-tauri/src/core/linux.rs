#[allow(unused_imports)]
use crate::core::{DatabaseInfo, DatabaseType};
#[allow(unused_imports)]
use anyhow::Result;
#[allow(unused_imports)]
use std::path::Path;

/// Options used when installing a database on Linux.
#[allow(dead_code)]
pub struct LinuxInstallOptions<'a> {
    pub version: Option<&'a str>,
    pub port: Option<u16>,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub auto_start: bool,
}

impl<'a> Default for LinuxInstallOptions<'a> {
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

#[cfg(target_os = "linux")]
pub use imp::*;

#[cfg(target_os = "linux")]
mod imp {
    use super::*;
    use crate::core::{utils, DatabaseStatus};
    use anyhow::{bail, Context};
    use reqwest::blocking::get;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    /// 安装数据库到 Linux
    pub fn install_database(
        db_type: &DatabaseType,
        storage_path: &Path,
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        match db_type {
            DatabaseType::Redis => install_redis(storage_path, options),
            DatabaseType::MySQL => install_mysql(storage_path, options),
            DatabaseType::MongoDB => install_mongodb(storage_path, options),
            DatabaseType::Qdrant => install_qdrant(storage_path, options),
            DatabaseType::Neo4j => install_neo4j(storage_path, options),
            DatabaseType::SurrealDB => install_surrealdb(storage_path, options),
            DatabaseType::SeekDB => install_seekdb(storage_path, options),
            DatabaseType::PostgreSQL => {
                bail!("PostgreSQL installation on Linux is not yet implemented via binary")
            }
        }
    }

    /// 启动数据库服务
    pub fn start_service(db_info: &DatabaseInfo) -> Result<()> {
        match db_info.db_type {
            DatabaseType::Redis => start_redis_process(db_info),
            DatabaseType::MySQL => start_mysql_process(db_info),
            DatabaseType::MongoDB => start_mongodb_process(db_info),
            DatabaseType::Qdrant => start_qdrant_process(db_info),
            DatabaseType::Neo4j => start_neo4j_process(db_info),
            DatabaseType::SurrealDB => start_surrealdb_process(db_info),
            DatabaseType::SeekDB => start_seekdb_process(db_info),
            DatabaseType::PostgreSQL => {
                bail!("PostgreSQL service management on Linux is not yet implemented")
            }
        }
    }

    /// 停止数据库服务
    pub fn stop_service(db_info: &DatabaseInfo) -> Result<()> {
        let pid_file = match db_info.db_type {
            DatabaseType::Redis => "redis.pid",
            DatabaseType::MySQL => "mysql.pid",
            DatabaseType::MongoDB => "mongodb.pid",
            DatabaseType::Qdrant => "qdrant.pid",
            DatabaseType::Neo4j => "neo4j.pid",
            DatabaseType::SurrealDB => "surrealdb.pid",
            DatabaseType::SeekDB => "seekdb.pid",
            DatabaseType::PostgreSQL => {
                bail!("PostgreSQL service management on Linux is not yet implemented")
            }
        };

        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join(pid_file);

        if pid_path.exists() {
            let pid_str = fs::read_to_string(&pid_path)?;
            let pid = pid_str.trim();

            // Linux: 使用 kill 停止进程
            Command::new("kill")
                .arg(pid)
                .output()
                .context("Failed to stop process via kill")?;

            let _ = fs::remove_file(&pid_path);
        }

        Ok(())
    }

    // --- Helper functions for binary installation ---

    fn download_file(url: &str, target_path: &Path) -> Result<()> {
        let response = get(url).context("Failed to download file")?;
        let mut content = response.bytes().context("Failed to read response bytes")?;
        fs::write(target_path, &mut content).context("Failed to write file")?;
        Ok(())
    }

    fn extract_archive(archive_path: &Path, target_dir: &Path) -> Result<()> {
        utils::ensure_dir(target_dir)?;

        // Use system tar command as it handles .tar.gz, .tar.xz etc.
        let output = Command::new("tar")
            .arg("-xf")
            .arg(archive_path)
            .arg("-C")
            .arg(target_dir)
            .output()
            .context("Failed to extract archive via tar command")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("Tar extraction failed: {}", err);
        }

        // Handle root directory if any
        move_from_root_if_needed(target_dir)?;

        Ok(())
    }

    fn move_from_root_if_needed(target_dir: &Path) -> Result<()> {
        let entries: Vec<_> = fs::read_dir(target_dir)?
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>();

        if entries.len() == 1 && entries[0].file_type()?.is_dir() {
            let root_dir = entries[0].path();
            for entry in fs::read_dir(&root_dir)? {
                let entry = entry?;
                let dest = target_dir.join(entry.file_name());
                fs::rename(entry.path(), dest)?;
            }
            fs::remove_dir(root_dir)?;
        }
        Ok(())
    }

    // --- Redis Implementation ---
    // Note: Redis on Linux is usually installed via package manager as per docs/linux.md
    // but for consistency with the "binary-first" approach, we might want to try binary if possible.
    // However, following the doc exactly:
    fn install_redis(
        storage_path: &Path,
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        bail!("Redis on Linux should be installed via package manager as per docs/linux.md. Automated binary install not yet supported.")
    }

    fn start_redis_process(db_info: &DatabaseInfo) -> Result<()> {
        bail!("Redis service management on Linux not yet implemented")
    }

    // --- Qdrant Implementation ---
    fn install_qdrant(
        storage_path: &Path,
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "qdrant");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "qdrant");
        let logs_dir = utils::get_db_log_path(storage_path, "qdrant");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("qdrant");
        let log_file = logs_dir.join("qdrant.log");
        let config_path =
            utils::get_db_config_path(storage_path, "qdrant").join("qdrant-config.yaml");
        utils::ensure_dir(config_path.parent().unwrap())?;

        if !binary_path.exists() {
            let arch = if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x86_64"
            };
            let url = format!(
                "https://github.com/qdrant/qdrant/releases/latest/download/qdrant-{}-unknown-linux-gnu.tar.gz",
                arch
            );
            let temp_archive = bin_dir.join("qdrant.tar.gz");
            download_file(&url, &temp_archive)?;
            extract_archive(&temp_archive, &bin_dir)?;
            let _ = fs::remove_file(&temp_archive);

            // Set execution permission
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        let port = options.port.unwrap_or(6333);
        let config_content = format!(
            "service:\n  port: {port}\n  grpc_port: {grpc_port}\nstorage:\n  path: {data_path}\nlogging:\n  level: INFO\n  file_path: {log_path}\n",
            port = port,
            grpc_port = port + 1,
            data_path = data_dir.to_string_lossy(),
            log_path = log_file.to_string_lossy()
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
        let binary_path = Path::new(&db_info.install_path).join("qdrant");
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
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "surrealdb");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "surrealdb");
        let logs_dir = utils::get_db_log_path(storage_path, "surrealdb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("surreal");
        let log_file = logs_dir.join("surreal.log");

        if !binary_path.exists() {
            // docs/linux.md suggests using curl -sSf https://install.surrealdb.com | sh
            // but we'll try to download the binary directly for more control
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "amd64"
            };
            let url = format!(
                "https://github.com/surrealdb/surrealdb/releases/latest/download/surreal-v2.1.4-linux-{}.tar.gz",
                arch
            );
            let temp_archive = bin_dir.join("surreal.tar.gz");
            download_file(&url, &temp_archive)?;
            extract_archive(&temp_archive, &bin_dir)?;
            let _ = fs::remove_file(&temp_archive);

            // The tar.gz usually contains the binary directly or in a subfolder.
            // Adjust path if needed.
            if !binary_path.exists() {
                // Check if it's in a subfolder like surreal-v2.1.4-linux-amd64/surreal
            }

            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
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
        let binary_path = Path::new(&db_info.install_path).join("surreal");
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("surrealdb.pid");

        let user = db_info.username.as_deref().unwrap_or("root");
        let pass = db_info.password.as_deref().unwrap_or("root");

        let child = Command::new(binary_path)
            .arg("start")
            .arg("--user")
            .arg(user)
            .arg("--pass")
            .arg(pass)
            .arg("--bind")
            .arg(format!("0.0.0.0:{}", db_info.port))
            .arg(format!("file://{}", data_dir.to_string_lossy()))
            .spawn()
            .context("Failed to start SurrealDB")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- SeekDB Implementation ---
    fn install_seekdb(
        storage_path: &Path,
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "seekdb");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "seekdb");
        let logs_dir = utils::get_db_log_path(storage_path, "seekdb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("seekdb");
        let log_file = logs_dir.join("seekdb.log");
        let config_path = utils::get_db_config_path(storage_path, "seekdb").join("seekdb.conf");
        utils::ensure_dir(config_path.parent().unwrap())?;

        if !binary_path.exists() {
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "amd64"
            };
            let url = format!(
                "https://github.com/seekdb/seekdb/releases/download/v0.1.0/seekdb_0.1.0_linux_{}.tar.gz",
                arch
            );
            let temp_archive = bin_dir.join("seekdb.tar.gz");
            download_file(&url, &temp_archive)?;
            extract_archive(&temp_archive, &bin_dir)?;
            let _ = fs::remove_file(&temp_archive);

            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        let port = options.port.unwrap_or(8080);
        let config_content = format!(
            "[server]\nport = {port}\nhost = 127.0.0.1\n\n[data]\ndir = {data_path}\n\n[logging]\nlevel = info\nfile = {log_path}\n",
            port = port,
            data_path = data_dir.to_string_lossy(),
            log_path = log_file.to_string_lossy()
        );
        fs::write(&config_path, config_content)?;

        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "SeekDB".to_string(),
            db_type: DatabaseType::SeekDB,
            version: "v0.1.0".to_string(),
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
        let binary_path = Path::new(&db_info.install_path).join("seekdb");
        let config_path = db_info.config.as_ref().context("Config path missing")?;
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("seekdb.pid");

        let child = Command::new(binary_path)
            .arg("-config")
            .arg(config_path)
            .spawn()
            .context("Failed to start SeekDB")?;

        fs::write(&pid_path, child.id().to_string())?;
        Ok(())
    }

    // --- MySQL, MongoDB, Neo4j, PostgreSQL ---
    // These require more complex setup (initialization, multiple files, etc.)
    // I will implement basic versions or bail if too complex for a single step.

    // --- MySQL Implementation ---

    fn install_mysql(
        storage_path: &Path,
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "mysql");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "mysql");
        let logs_dir = utils::get_db_log_path(storage_path, "mysql");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("bin").join("mysqld");
        let log_file = logs_dir.join("mysqld.log");
        let config_path = utils::get_db_config_path(storage_path, "mysql").join("my.cnf");
        utils::ensure_dir(config_path.parent().unwrap())?;

        if !binary_path.exists() {
            let arch = if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x86_64"
            };
            let url = format!(
                "https://cdn.mysql.com/Downloads/MySQL-8.4/mysql-8.4.0-linux-glibc2.28-{}.tar.xz",
                arch
            );
            let temp_archive = bin_dir.join("mysql.tar.xz");
            download_file(&url, &temp_archive)?;
            extract_archive(&temp_archive, &bin_dir)?;
            let _ = fs::remove_file(&temp_archive);
        }

        // Initialize MySQL
        if !data_dir.join("mysql").exists() {
            let output = Command::new(&binary_path)
                .arg("--initialize-insecure") // Use insecure for easier setup
                .arg(format!("--datadir={}", data_dir.to_string_lossy()))
                .arg(format!("--basedir={}", bin_dir.to_string_lossy()))
                .output()
                .context("Failed to initialize MySQL")?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                bail!("MySQL initialization failed: {}", err);
            }
        }

        let port = options.port.unwrap_or(3306);
        let config_content = format!(
            "[mysqld]\ndatadir={data_path}\nbasedir={base_path}\nsocket=/tmp/mysql.sock\nport={port}\nlog-error={log_path}\npid-file={pid_path}\n",
            data_path = data_dir.to_string_lossy(),
            base_path = bin_dir.to_string_lossy(),
            port = port,
            log_path = log_file.to_string_lossy(),
            pid_path = data_dir.join("mysql.pid").to_string_lossy()
        );
        fs::write(&config_path, config_content)?;

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
            config: Some(config_path.to_string_lossy().to_string()),
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
        let binary_path = Path::new(&db_info.install_path).join("bin").join("mysqld");
        let config_path = db_info.config.as_ref().context("Config path missing")?;

        let child = Command::new(binary_path)
            .arg(format!("--defaults-file={}", config_path))
            .spawn()
            .context("Failed to start MySQL")?;

        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("mysql.pid");
        fs::write(&pid_path, child.id().to_string())?;

        Ok(())
    }

    // --- MongoDB Implementation ---

    fn install_mongodb(
        storage_path: &Path,
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "mongodb");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "mongodb");
        let logs_dir = utils::get_db_log_path(storage_path, "mongodb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let binary_path = bin_dir.join("bin").join("mongod");
        let log_file = logs_dir.join("mongod.log");
        let config_path = utils::get_db_config_path(storage_path, "mongodb").join("mongod.conf");
        utils::ensure_dir(config_path.parent().unwrap())?;

        if !binary_path.exists() {
            let arch = if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x86_64"
            };
            let url = format!(
                "https://fastdl.mongodb.org/linux/mongodb-linux-{}-ubuntu2004-7.0.9.tgz",
                arch
            );
            let temp_archive = bin_dir.join("mongodb.tgz");
            download_file(&url, &temp_archive)?;
            extract_archive(&temp_archive, &bin_dir)?;
            let _ = fs::remove_file(&temp_archive);
        }

        let port = options.port.unwrap_or(27017);
        let config_content = format!(
            "systemLog:\n  destination: file\n  path: \"{log_path}\"\n  logAppend: true\nstorage:\n  dbPath: \"{data_path}\"\n  engine: wiredTiger\nnet:\n  port: {port}\n  bindIp: 127.0.0.1\nprocessManagement:\n  fork: true\n  pidFilePath: {pid_path}\n",
            log_path = log_file.to_string_lossy(),
            data_path = data_dir.to_string_lossy(),
            port = port,
            pid_path = data_dir.join("mongodb.pid").to_string_lossy()
        );
        fs::write(&config_path, config_content)?;

        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: "MongoDB".to_string(),
            db_type: DatabaseType::MongoDB,
            version: "7.0.9".to_string(),
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
            start_mongodb_process(&db_info)?;
            db_info.status = DatabaseStatus::Running;
        }

        Ok(db_info)
    }

    fn start_mongodb_process(db_info: &DatabaseInfo) -> Result<()> {
        let binary_path = Path::new(&db_info.install_path).join("bin").join("mongod");
        let config_path = db_info.config.as_ref().context("Config path missing")?;

        let output = Command::new(binary_path)
            .arg("--config")
            .arg(config_path)
            .output()
            .context("Failed to start MongoDB")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("MongoDB failed to start: {}", err);
        }

        Ok(())
    }

    fn install_neo4j(
        storage_path: &Path,
        options: &LinuxInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        bail!("Neo4j binary installation on Linux is complex and not yet fully automated. Please refer to docs/linux.md")
    }

    fn start_neo4j_process(db_info: &DatabaseInfo) -> Result<()> {
        bail!("Neo4j service management on Linux not yet implemented")
    }
}
