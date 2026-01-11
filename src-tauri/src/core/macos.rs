use crate::core::{utils, DatabaseInfo, DatabaseStatus, DatabaseType};
use anyhow::Result;

/// Options used when installing a database via Homebrew.
pub struct HomebrewInstallOptions<'a> {
    pub version: Option<&'a str>,
    pub port: Option<u16>,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub auto_start: bool,
}

impl<'a> Default for HomebrewInstallOptions<'a> {
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

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use anyhow::{anyhow, bail, Context};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const HOMEBREW_INSTALL_URL: &str =
        "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh";

    pub fn install_database_via_homebrew(
        db_type: &DatabaseType,
        storage_path: &Path,
        options: &HomebrewInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let brew = Homebrew::bootstrap()?;
        let recipe = HomebrewDatabaseRecipe::resolve(db_type)?;

        brew.ensure_formula(recipe.tap, recipe.formula)?;

        let configured = configure_database(
            db_type,
            &brew,
            storage_path,
            options.port.unwrap_or(recipe.port),
        )?;
        let data_path = utils::get_db_data_path(storage_path, db_type.as_str());
        let install_prefix = brew.prefix(Some(recipe.formula))?;

        let version = brew
            .formula_version(recipe.formula)?
            .or_else(|| options.version.map(|v| v.to_string()))
            .unwrap_or_else(|| "latest".to_string());

        let status = if options.auto_start {
            brew.restart_service(recipe.service_name)?;
            DatabaseStatus::Running
        } else {
            DatabaseStatus::Stopped
        };

        Ok(DatabaseInfo {
            id: utils::generate_id(),
            name: db_type.display_name().to_string(),
            db_type: db_type.clone(),
            version,
            install_path: install_prefix.to_string_lossy().to_string(),
            data_path: data_path.to_string_lossy().to_string(),
            log_path: configured.log_path.to_string_lossy().to_string(),
            port: options.port.unwrap_or(recipe.port),
            username: resolve_username(db_type, options.username),
            password: options.password.map(|s| s.to_string()),
            config: Some(configured.config_path.to_string_lossy().to_string()),
            status,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        })
    }

    pub fn start_service_for_database(db_info: &DatabaseInfo) -> Result<()> {
        let brew = Homebrew::bootstrap()?;
        let recipe = HomebrewDatabaseRecipe::resolve(&db_info.db_type)?;
        brew.start_service(recipe.service_name)
    }

    pub fn stop_service_for_database(db_info: &DatabaseInfo) -> Result<()> {
        let brew = Homebrew::bootstrap()?;
        let recipe = HomebrewDatabaseRecipe::resolve(&db_info.db_type)?;
        brew.stop_service(recipe.service_name)
    }

    struct ConfiguredPaths {
        config_path: PathBuf,
        log_path: PathBuf,
    }

    fn configure_database(
        db_type: &DatabaseType,
        brew: &Homebrew,
        storage_path: &Path,
        port: u16,
    ) -> Result<ConfiguredPaths> {
        match db_type {
            DatabaseType::Redis => configure_redis(brew, storage_path),
            DatabaseType::MySQL => configure_mysql(brew, storage_path, port),
            DatabaseType::PostgreSQL => configure_postgresql(brew, storage_path, port),
            DatabaseType::MongoDB => configure_mongodb(brew, storage_path, port),
            DatabaseType::Neo4j => configure_neo4j(brew, storage_path, port),
            DatabaseType::Qdrant => configure_qdrant(brew, storage_path, port),
            DatabaseType::SeekDB => configure_seekdb(brew, storage_path, port),
            DatabaseType::SurrealDB => configure_surrealdb(brew, storage_path, port),
        }
    }

    fn configure_redis(brew: &Homebrew, storage_path: &Path) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("redis"))?;
        let conf_path = prefix.join("etc").join("redis.conf");
        if !conf_path.exists() {
            bail!("Redis config not found at {}", conf_path.display());
        }

        let data_dir = utils::get_db_data_path(storage_path, "redis");
        let logs_dir = utils::get_db_log_path(storage_path, "redis");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("redis.log");

        let contents = fs::read_to_string(&conf_path)
            .with_context(|| format!("Failed to read redis.conf at {}", conf_path.display()))?;
        let mut rewritten = String::with_capacity(contents.len() + 200);
        for line in contents.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("dir ") {
                rewritten.push_str(&format!("dir {}\n", data_dir.display()));
            } else if trimmed.starts_with("logfile ") {
                rewritten.push_str(&format!("logfile {}\n", log_file.display()));
            } else {
                rewritten.push_str(line);
                rewritten.push('\n');
            }
        }
        fs::write(&conf_path, rewritten).with_context(|| "Failed to update redis.conf")?;

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn configure_mysql(brew: &Homebrew, storage_path: &Path, port: u16) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("mysql"))?;
        let etc_dir = prefix.join("etc");
        utils::ensure_dir(&etc_dir)?;
        let conf_path = etc_dir.join("my.cnf");

        let data_dir = utils::get_db_data_path(storage_path, "mysql");
        let logs_dir = utils::get_db_log_path(storage_path, "mysql");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;

        let log_file = logs_dir.join("mysqld.log");
        let pid_file = data_dir.join("mysqld.pid");
        let socket_file = data_dir.join("mysql.sock");
        let user = std::env::var("USER").unwrap_or_else(|_| "local".to_string());

        let config_content = format!(
			"[mysqld]\n	datadir = {datadir}\n\tsocket = {socket}\n\tlog-error = {log_error}\n\tpid-file = {pid_file}\n\tport = {port}\n\tuser = {user}\n[client]\n\tsocket = {socket}\n\tport = {port}\n",
			datadir = data_dir.display(),
			socket = socket_file.display(),
			log_error = log_file.display(),
			pid_file = pid_file.display(),
			port = port,
			user = user
		);
        fs::write(&conf_path, config_content).with_context(|| "Failed to write my.cnf")?;

        initialize_mysql_data_dir(&prefix, &data_dir, &user)?;

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn initialize_mysql_data_dir(prefix: &Path, data_dir: &Path, user: &str) -> Result<()> {
        if !data_dir.exists() {
            utils::ensure_dir(data_dir)?;
        }

        let mut entries = fs::read_dir(data_dir)?;
        if entries.next().is_some() {
            return Ok(());
        }

        let mysqld_path = prefix.join("bin").join("mysqld");
        if !mysqld_path.exists() {
            bail!("mysqld binary not found at {}", mysqld_path.display());
        }

        let status = Command::new(&mysqld_path)
            .arg("--initialize-insecure")
            .arg(format!("--user={}", user))
            .arg(format!("--datadir={}", data_dir.display()))
            .status()
            .with_context(|| "Failed to initialize MySQL data directory")?;

        if !status.success() {
            bail!("mysqld initialization failed: {}", status);
        }

        Ok(())
    }

    fn configure_postgresql(
        brew: &Homebrew,
        storage_path: &Path,
        _port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("postgresql"))?;
        let data_dir = utils::get_db_data_path(storage_path, "postgresql");
        let logs_dir = utils::get_db_log_path(storage_path, "postgresql");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("postgresql.log");

        Ok(ConfiguredPaths {
            config_path: prefix.join("etc").join("postgresql.conf"),
            log_path: log_file,
        })
    }

    fn configure_mongodb(
        brew: &Homebrew,
        storage_path: &Path,
        _port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("mongodb-community@7.0"))?;
        let data_dir = utils::get_db_data_path(storage_path, "mongodb");
        let logs_dir = utils::get_db_log_path(storage_path, "mongodb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("mongod.log");
        let conf_path = prefix.join("etc").join("mongod.conf");

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn configure_neo4j(
        brew: &Homebrew,
        storage_path: &Path,
        _port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("neo4j"))?;
        let data_dir = utils::get_db_data_path(storage_path, "neo4j");
        let logs_dir = utils::get_db_log_path(storage_path, "neo4j");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("neo4j.log");
        let conf_path = prefix.join("etc").join("neo4j.conf");

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn configure_qdrant(
        brew: &Homebrew,
        storage_path: &Path,
        _port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("qdrant"))?;
        let data_dir = utils::get_db_data_path(storage_path, "qdrant");
        let logs_dir = utils::get_db_log_path(storage_path, "qdrant");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("qdrant.log");
        let conf_path = prefix.join("etc").join("config.yaml");

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn configure_seekdb(
        brew: &Homebrew,
        storage_path: &Path,
        _port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("seekdb"))?;
        let data_dir = utils::get_db_data_path(storage_path, "seekdb");
        let logs_dir = utils::get_db_log_path(storage_path, "seekdb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("seekdb.log");
        let conf_path = prefix.join("etc").join("seekdb.conf");

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn configure_surrealdb(
        brew: &Homebrew,
        storage_path: &Path,
        _port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("surreal"))?;
        let data_dir = utils::get_db_data_path(storage_path, "surrealdb");
        let logs_dir = utils::get_db_log_path(storage_path, "surrealdb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("surrealdb.log");
        let conf_path = prefix.join("etc").join("surrealdb.yaml");

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn resolve_username(db_type: &DatabaseType, override_username: Option<&str>) -> Option<String> {
        if let Some(name) = override_username {
            return Some(name.to_string());
        }
        match db_type {
            DatabaseType::MySQL => Some("root".to_string()),
            DatabaseType::PostgreSQL => Some("postgres".to_string()),
            _ => None,
        }
    }

    struct Homebrew {
        bin_path: PathBuf,
    }

    impl Homebrew {
        fn bootstrap() -> Result<Self> {
            if let Some(existing) = Self::detect()? {
                return Ok(existing);
            }
            install_homebrew_via_script()?;
            Self::detect()?
                .ok_or_else(|| anyhow!("Homebrew installation completed but binary not found"))
        }

        fn detect() -> Result<Option<Self>> {
            if let Some(path) = Self::which_brew()? {
                return Ok(Some(Self { bin_path: path }));
            }
            for candidate in ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"] {
                let candidate_path = Path::new(candidate);
                if candidate_path.exists() {
                    return Ok(Some(Self {
                        bin_path: candidate_path.to_path_buf(),
                    }));
                }
            }
            Ok(None)
        }

        fn which_brew() -> Result<Option<PathBuf>> {
            let output = Command::new("which").arg("brew").output();
            match output {
                Ok(output) if output.status.success() => {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if path.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(PathBuf::from(path)))
                    }
                }
                _ => Ok(None),
            }
        }

        fn ensure_formula(&self, tap: Option<&str>, formula: &str) -> Result<()> {
            if let Some(tap) = tap {
                if !self.is_tap_exists(tap)? {
                    self.run(&["tap", tap])?;
                }
            }

            if self.is_formula_installed(formula)? {
                return Ok(());
            }
            self.run(&["install", formula]).map(|_| ())
        }

        fn is_tap_exists(&self, tap: &str) -> Result<bool> {
            let output = Command::new(&self.bin_path)
                .args(["tap"])
                .output()
                .with_context(|| "Failed to list taps")?;
            let taps = String::from_utf8_lossy(&output.stdout);
            Ok(taps.lines().any(|line| line.contains(tap)))
        }

        fn is_formula_installed(&self, formula: &str) -> Result<bool> {
            let output = Command::new(&self.bin_path)
                .args(["list", "--versions", formula])
                .output()
                .with_context(|| format!("Failed to check {} installation", formula))?;
            Ok(output.status.success()
                && !String::from_utf8_lossy(&output.stdout).trim().is_empty())
        }

        fn run(&self, args: &[&str]) -> Result<String> {
            let output = Command::new(&self.bin_path)
                .args(args)
                .output()
                .with_context(|| format!("Failed to run brew {}", args.join(" ")))?;
            if !output.status.success() {
                bail!(
                    "brew {} failed: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }

        fn prefix(&self, formula: Option<&str>) -> Result<PathBuf> {
            let mut args = vec!["--prefix"];
            if let Some(name) = formula {
                args.push(name);
            }
            let output = self.run(&args)?;
            Ok(PathBuf::from(output.trim()))
        }

        fn formula_version(&self, formula: &str) -> Result<Option<String>> {
            let output = self.run(&["list", "--versions", formula])?;
            let trimmed = output.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let mut parts = trimmed.split_whitespace();
            let _ = parts.next();
            Ok(parts.next().map(|v| v.to_string()))
        }

        fn start_service(&self, service: &str) -> Result<()> {
            self.run(&["services", "start", service]).map(|_| ())
        }

        fn stop_service(&self, service: &str) -> Result<()> {
            let output = Command::new(&self.bin_path)
                .args(["services", "stop", service])
                .output()
                .with_context(|| format!("Failed to stop brew service {}", service))?;
            if output.status.success() {
                return Ok(());
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Service `") && stderr.contains("is not started") {
                return Ok(());
            }
            bail!("brew services stop {} failed: {}", service, stderr);
        }

        fn restart_service(&self, service: &str) -> Result<()> {
            let _ = self.stop_service(service);
            self.start_service(service)
        }
    }

    struct HomebrewDatabaseRecipe {
        db_type: DatabaseType,
        tap: Option<&'static str>,
        formula: &'static str,
        service_name: &'static str,
        port: u16,
    }

    impl HomebrewDatabaseRecipe {
        fn resolve(db_type: &DatabaseType) -> Result<Self> {
            match db_type {
                DatabaseType::Redis => Ok(Self {
                    db_type: DatabaseType::Redis,
                    tap: None,
                    formula: "redis",
                    service_name: "redis",
                    port: 6379,
                }),
                DatabaseType::MySQL => Ok(Self {
                    db_type: DatabaseType::MySQL,
                    tap: None,
                    formula: "mysql",
                    service_name: "mysql",
                    port: 3306,
                }),
                DatabaseType::PostgreSQL => Ok(Self {
                    db_type: DatabaseType::PostgreSQL,
                    tap: None,
                    formula: "postgresql",
                    service_name: "postgresql",
                    port: 5432,
                }),
                DatabaseType::MongoDB => Ok(Self {
                    db_type: DatabaseType::MongoDB,
                    tap: Some("mongodb/brew"),
                    formula: "mongodb-community@7.0",
                    service_name: "mongodb-community@7.0",
                    port: 27017,
                }),
                DatabaseType::Neo4j => Ok(Self {
                    db_type: DatabaseType::Neo4j,
                    tap: None,
                    formula: "neo4j",
                    service_name: "neo4j",
                    port: 7474,
                }),
                DatabaseType::Qdrant => Ok(Self {
                    db_type: DatabaseType::Qdrant,
                    tap: None,
                    formula: "qdrant",
                    service_name: "qdrant",
                    port: 6333,
                }),
                DatabaseType::SeekDB => Ok(Self {
                    db_type: DatabaseType::SeekDB,
                    tap: Some("seekdb/tap/seekdb"),
                    formula: "seekdb",
                    service_name: "seekdb",
                    port: 8080,
                }),
                DatabaseType::SurrealDB => Ok(Self {
                    db_type: DatabaseType::SurrealDB,
                    tap: Some("surrealdb/tap/surreal"),
                    formula: "surreal",
                    service_name: "surreal",
                    port: 8000,
                }),
            }
        }
    }

    fn install_homebrew_via_script() -> Result<()> {
        let installer_path = std::env::temp_dir().join("homebrew-install.sh");
        let download_cmd = format!(
            "curl -fsSL {} -o {}",
            HOMEBREW_INSTALL_URL,
            installer_path.display()
        );
        run_shell_command(&download_cmd)?;

        let install_cmd = format!("NONINTERACTIVE=1 /bin/bash {}", installer_path.display());
        run_shell_command(&install_cmd)?;

        let _ = fs::remove_file(&installer_path);
        Ok(())
    }

    fn run_shell_command(command: &str) -> Result<()> {
        let status = Command::new("/bin/bash")
            .arg("-c")
            .arg(command)
            .status()
            .with_context(|| format!("Failed to execute shell command: {}", command))?;
        if !status.success() {
            bail!("Command `{}` exited with status {}", command, status);
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub use imp::{
    install_database_via_homebrew, start_service_for_database, stop_service_for_database,
};

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::*;
    use anyhow::{bail, Result};
    use std::path::Path;

    pub fn install_database_via_homebrew(
        _db_type: &DatabaseType,
        _storage_path: &Path,
        _options: &HomebrewInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        bail!("Homebrew workflow is only available on macOS");
    }

    pub fn start_service_for_database(_db_info: &DatabaseInfo) -> Result<()> {
        bail!("Homebrew workflow is only available on macOS");
    }

    pub fn stop_service_for_database(_db_info: &DatabaseInfo) -> Result<()> {
        bail!("Homebrew workflow is only available on macOS");
    }
}

#[cfg(not(target_os = "macos"))]
pub use imp::{
    install_database_via_homebrew, start_service_for_database, stop_service_for_database,
};
