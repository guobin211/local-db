use crate::core::{utils, DatabaseInfo, DatabaseStatus, DatabaseType};
use anyhow::Result;
use reqwest::blocking::get;

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

    /// 获取 Homebrew 已安装的数据库类型列表
    pub fn get_installed_databases_from_homebrew() -> Vec<DatabaseType> {
        if let Ok(brew) = Homebrew::bootstrap() {
            let mut installed = Vec::new();

            // 检查每种数据库类型的 Homebrew formula 是否已安装
            let formulas = vec![
                ("redis", DatabaseType::Redis),
                ("mysql", DatabaseType::MySQL),
                ("postgresql", DatabaseType::PostgreSQL),
                ("mongodb-community@7.0", DatabaseType::MongoDB),
                ("surreal", DatabaseType::SurrealDB),
            ];

            for (formula, db_type) in formulas {
                if brew.is_formula_installed(formula).unwrap_or(false) {
                    installed.push(db_type);
                }
            }

            installed
        } else {
            Vec::new()
        }
    }

    /// 检查 Homebrew 服务的运行状态
    pub fn get_homebrew_service_status(service: &str) -> Option<bool> {
        if let Ok(brew) = Homebrew::bootstrap() {
            brew.get_service_status(service).ok()
        } else {
            None
        }
    }

    /// 获取所有 Homebrew 服务的运行状态（一次性调用）
    /// 返回 HashMap<服务名, 是否运行>
    pub fn get_all_homebrew_services_status() -> std::collections::HashMap<String, bool> {
        let mut result = std::collections::HashMap::new();

        if let Ok(brew) = Homebrew::bootstrap() {
            if let Ok(output) = brew.list_services() {
                // 解析 brew services list 的输出
                // 输出格式示例：
                // Name              Status       User File
                // mongodb-community none
                // mysql@8.4         started      guobin ~/Library/LaunchAgents/homebrew.mxcl.mysql@8.4.plist
                // postgresql@18     stopped
                // redis             started      guobin ~/Library/LaunchAgents/homebrew.mxcl.redis.plist
                for line in output.lines().skip(1) {
                    // 跳过标题行
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let service_name = parts[0].to_string();
                        let status = parts[1];
                        let is_running = status == "started";
                        result.insert(service_name, is_running);
                    }
                }
            }
        }

        result
    }

    pub fn install_database_via_homebrew(
        db_type: &DatabaseType,
        storage_path: &Path,
        options: &HomebrewInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        // Qdrant 使用二进制安装，不通过 Homebrew
        if *db_type == DatabaseType::Qdrant {
            return install_qdrant_binary(storage_path, options);
        }

        // SurrealDB 通过 Homebrew 安装，但使用直接进程启动
        if *db_type == DatabaseType::SurrealDB {
            return install_surrealdb_via_homebrew(storage_path, options);
        }

        let brew = Homebrew::bootstrap()?;
        let recipe = HomebrewDatabaseRecipe::resolve(db_type)?;

        // 检查是否已安装，如果已安装则跳过安装步骤
        let already_installed = brew.is_formula_installed(recipe.formula)?;
        if !already_installed {
            // 未安装，执行安装
            brew.ensure_formula(recipe.tap, recipe.formula)?;
        }

        // 继续配置流程（无论是否已安装都需要确保配置正确）
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
            password: resolve_password(db_type, options.password),
            config: Some(configured.config_path.to_string_lossy().to_string()),
            status,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        })
    }

    /// 通过 Homebrew 安装 SurrealDB，但使用直接进程启动
    fn install_surrealdb_via_homebrew(
        storage_path: &Path,
        options: &HomebrewInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let brew = Homebrew::bootstrap()?;
        let tap = Some("surrealdb/tap");
        let formula = "surreal";
        let default_port = 8000u16;

        // 检查是否已安装，如果已安装则跳过安装步骤
        let already_installed = brew.is_formula_installed(formula)?;
        if !already_installed {
            brew.ensure_formula(tap, formula)?;
        }

        // 配置 SurrealDB
        let port = options.port.unwrap_or(default_port);
        let configured = configure_surrealdb(&brew, storage_path, port)?;
        let data_path = utils::get_db_data_path(storage_path, "surrealdb");
        let install_prefix = brew.prefix(Some(formula))?;

        let version = brew
            .formula_version(formula)?
            .or_else(|| options.version.map(|v| v.to_string()))
            .unwrap_or_else(|| "latest".to_string());

        // 创建 DatabaseInfo
        let mut db_info = DatabaseInfo {
            id: utils::generate_id(),
            name: DatabaseType::SurrealDB.display_name().to_string(),
            db_type: DatabaseType::SurrealDB,
            version,
            install_path: install_prefix.to_string_lossy().to_string(),
            data_path: data_path.to_string_lossy().to_string(),
            log_path: configured.log_path.to_string_lossy().to_string(),
            port,
            username: resolve_username(&DatabaseType::SurrealDB, options.username),
            password: resolve_password(&DatabaseType::SurrealDB, options.password),
            config: Some(configured.config_path.to_string_lossy().to_string()),
            status: DatabaseStatus::Stopped,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        };

        // 如果需要自动启动，使用直接进程启动
        if options.auto_start {
            if let Err(e) = start_surrealdb_process(&db_info) {
                eprintln!("Warning: Failed to start SurrealDB: {}", e);
            } else {
                db_info.status = DatabaseStatus::Running;
            }
        }

        Ok(db_info)
    }

    /// 获取当前系统架构对应的 Qdrant 二进制文件名
    fn get_qdrant_binary_name() -> &'static str {
        #[cfg(target_arch = "aarch64")]
        {
            "qdrant-aarch64-apple-darwin.tar.gz"
        }
        #[cfg(target_arch = "x86_64")]
        {
            "qdrant-x86_64-apple-darwin.tar.gz"
        }
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            compile_error!("Unsupported architecture for Qdrant")
        }
    }

    /// 构建 Qdrant 下载 URL
    fn get_qdrant_download_url(version: Option<&str>) -> String {
        let binary_name = get_qdrant_binary_name();
        match version {
            Some(v) => {
                let ver = if v.starts_with('v') {
                    v.to_string()
                } else {
                    format!("v{}", v)
                };
                format!(
                    "https://github.com/qdrant/qdrant/releases/download/{}/{}",
                    ver, binary_name
                )
            }
            None => format!(
                "https://github.com/qdrant/qdrant/releases/latest/download/{}",
                binary_name
            ),
        }
    }

    /// 安装 Qdrant 二进制文件
    fn install_qdrant_binary(
        storage_path: &Path,
        options: &HomebrewInstallOptions<'_>,
    ) -> Result<DatabaseInfo> {
        let bin_dir = utils::get_db_bin_path(storage_path, "qdrant");
        utils::ensure_dir(&bin_dir)?;

        let data_dir = utils::get_db_data_path(storage_path, "qdrant");
        let logs_dir = utils::get_db_log_path(storage_path, "qdrant");
        let config_dir = utils::get_db_config_path(storage_path, "qdrant");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        utils::ensure_dir(&config_dir)?;

        let log_file = logs_dir.join("qdrant.log");
        let config_path = config_dir.join("config.yaml");
        let binary_path = bin_dir.join("qdrant");

        // 下载 Qdrant 二进制文件 (tar.gz 格式)
        let binary_url = get_qdrant_download_url(options.version);

        if !binary_path.exists() {
            eprintln!("Downloading Qdrant from {}...", binary_url);
            let response = get(&binary_url).context("Failed to fetch Qdrant archive")?;

            if !response.status().is_success() {
                bail!(
                    "Failed to download Qdrant archive: HTTP {}",
                    response.status()
                );
            }

            let bytes = response.bytes().context("Failed to read Qdrant archive")?;

            // 解压 tar.gz 文件
            eprintln!("Extracting Qdrant binary...");
            let tar_gz = std::io::Cursor::new(bytes);
            let tar = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);

            // 解压到 bin 目录
            for entry in archive.entries().context("Failed to read tar entries")? {
                let mut entry = entry.context("Failed to read tar entry")?;
                let path = entry.path().context("Failed to get entry path")?;

                // 只提取 qdrant 可执行文件
                if let Some(file_name) = path.file_name() {
                    if file_name == "qdrant" {
                        entry
                            .unpack(&binary_path)
                            .context("Failed to extract qdrant binary")?;
                        break;
                    }
                }
            }

            // 确保二进制文件存在
            if !binary_path.exists() {
                bail!("Qdrant binary not found in archive");
            }

            // 设置可执行权限
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&binary_path)
                    .context("Failed to get binary permissions")?
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&binary_path, perms)
                    .context("Failed to set binary permissions")?;
            }
        }

        let port = options.port.unwrap_or(6333);
        let grpc_port = port + 1; // gRPC port = HTTP port + 1

        // 创建配置文件 (正确的 YAML 格式)
        let config_content = format!(
            r#"# Qdrant configuration for local-db
service:
  host: 0.0.0.0
  http_port: {port}
  grpc_port: {grpc_port}

log_level: INFO

storage:
  storage_path: {data_path}

telemetry_disabled: true
"#,
            port = port,
            grpc_port = grpc_port,
            data_path = data_dir.display()
        );
        fs::write(&config_path, config_content).context("Failed to write Qdrant config")?;

        // 获取版本号
        let version = options
            .version
            .map(|v| v.to_string())
            .unwrap_or_else(|| "latest".to_string());

        // 如果需要自动启动，启动 Qdrant
        let status = if options.auto_start {
            if let Err(e) = start_qdrant_process(&binary_path, &config_path, &data_dir) {
                eprintln!("Warning: Failed to start Qdrant: {}", e);
                DatabaseStatus::Stopped
            } else {
                DatabaseStatus::Running
            }
        } else {
            DatabaseStatus::Stopped
        };

        Ok(DatabaseInfo {
            id: utils::generate_id(),
            name: "Qdrant".to_string(),
            db_type: DatabaseType::Qdrant,
            version,
            install_path: bin_dir.to_string_lossy().to_string(),
            data_path: data_dir.to_string_lossy().to_string(),
            log_path: log_file.to_string_lossy().to_string(),
            port,
            username: None,
            password: options.password.map(|s| s.to_string()),
            config: Some(config_path.to_string_lossy().to_string()),
            status,
            auto_start: options.auto_start,
            pid: None,
            created_at: utils::get_timestamp(),
            updated_at: utils::get_timestamp(),
        })
    }

    /// 启动 Qdrant 进程
    fn start_qdrant_process(binary_path: &Path, config_path: &Path, data_dir: &Path) -> Result<()> {
        // 检查二进制文件是否存在
        if !binary_path.exists() {
            bail!("Qdrant binary not found at {}", binary_path.display());
        }

        // 检查是否已经在运行
        let pid_path = data_dir.join("qdrant.pid");
        if pid_path.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_path) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    // 检查进程是否存在
                    #[cfg(unix)]
                    {
                        use nix::sys::signal::{kill, Signal};
                        use nix::unistd::Pid;
                        if kill(Pid::from_raw(pid), Signal::SIGCONT).is_ok() {
                            // 进程仍在运行
                            return Ok(());
                        }
                    }
                }
            }
            // PID 文件存在但进程不存在，删除旧的 PID 文件
            let _ = fs::remove_file(&pid_path);
        }

        // 使用配置文件启动 Qdrant
        let child = Command::new(binary_path)
            .arg("--config-path")
            .arg(config_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("Failed to spawn Qdrant process")?;

        // 保存 PID 以便后续管理
        fs::write(&pid_path, child.id().to_string()).context("Failed to write PID file")?;

        // 等待一小段时间，确保进程启动
        std::thread::sleep(std::time::Duration::from_millis(500));

        // 验证进程是否成功启动
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            if kill(Pid::from_raw(child.id() as i32), Signal::SIGCONT).is_err() {
                let _ = fs::remove_file(&pid_path);
                bail!("Qdrant process failed to start");
            }
        }

        Ok(())
    }

    /// 停止 Qdrant 进程
    pub fn stop_qdrant_process(db_info: &DatabaseInfo) -> Result<()> {
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("qdrant.pid");

        if pid_path.exists() {
            let pid_str = fs::read_to_string(&pid_path)
                .context("Failed to read PID file")?
                .trim()
                .to_string();

            let pid: u32 = pid_str.parse().context("Invalid PID format")?;

            #[cfg(unix)]
            {
                // Unix: 使用 SIGTERM
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
                    .context("Failed to send SIGTERM to Qdrant")?;

                // 等待进程结束
                std::thread::sleep(std::time::Duration::from_secs(2));
            }

            #[cfg(windows)]
            {
                // Windows: 使用 taskkill
                Command::new("taskkill")
                    .arg("/PID")
                    .arg(&pid_str)
                    .arg("/F")
                    .output()
                    .context("Failed to stop Qdrant process")?;
            }

            let _ = fs::remove_file(&pid_path);
        }

        Ok(())
    }

    /// 启动 SurrealDB 进程
    fn start_surrealdb_process(db_info: &DatabaseInfo) -> Result<()> {
        let brew = Homebrew::bootstrap()?;
        let binary_path = brew.prefix(Some("surreal"))?.join("bin").join("surreal");

        if !binary_path.exists() {
            bail!("SurrealDB binary not found at {}", binary_path.display());
        }

        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("surrealdb.pid");

        // 检查是否已经在运行
        if pid_path.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_path) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    #[cfg(unix)]
                    {
                        use nix::sys::signal::{kill, Signal};
                        use nix::unistd::Pid;
                        if kill(Pid::from_raw(pid), Signal::SIGCONT).is_ok() {
                            // 进程仍在运行
                            return Ok(());
                        }
                    }
                }
            }
            // PID 文件存在但进程不存在，删除旧的 PID 文件
            let _ = fs::remove_file(&pid_path);
        }

        let logs_dir = Path::new(&db_info.log_path).parent().unwrap_or(data_dir);
        let log_file = logs_dir.join("surrealdb.log");

        // 启动 SurrealDB: surreal start --bind 0.0.0.0:port rocksdb://data_path
        let child = Command::new(&binary_path)
            .arg("start")
            .arg("--bind")
            .arg(format!("0.0.0.0:{}", db_info.port))
            .arg("--log")
            .arg("info")
            .arg("--user")
            .arg(db_info.username.as_deref().unwrap_or("admin"))
            .arg("--pass")
            .arg(db_info.password.as_deref().unwrap_or("admin888"))
            .arg(format!("rocksdb://{}", data_dir.display()))
            .stdin(std::process::Stdio::null())
            .stdout(
                fs::File::create(&log_file)
                    .map(std::process::Stdio::from)
                    .unwrap_or(std::process::Stdio::null()),
            )
            .stderr(
                fs::File::options()
                    .append(true)
                    .open(&log_file)
                    .map(std::process::Stdio::from)
                    .unwrap_or(std::process::Stdio::null()),
            )
            .spawn()
            .context("Failed to spawn SurrealDB process")?;

        // 保存 PID
        fs::write(&pid_path, child.id().to_string()).context("Failed to write PID file")?;

        // 等待一小段时间，确保进程启动
        std::thread::sleep(std::time::Duration::from_millis(500));

        // 验证进程是否成功启动
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            if kill(Pid::from_raw(child.id() as i32), Signal::SIGCONT).is_err() {
                let _ = fs::remove_file(&pid_path);
                bail!("SurrealDB process failed to start");
            }
        }

        Ok(())
    }

    /// 停止 SurrealDB 进程
    pub fn stop_surrealdb_process(db_info: &DatabaseInfo) -> Result<()> {
        let data_dir = Path::new(&db_info.data_path);
        let pid_path = data_dir.join("surrealdb.pid");

        if pid_path.exists() {
            let pid_str = fs::read_to_string(&pid_path)
                .context("Failed to read PID file")?
                .trim()
                .to_string();

            let pid: u32 = pid_str.parse().context("Invalid PID format")?;

            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
                    .context("Failed to send SIGTERM to SurrealDB")?;

                // 等待进程结束
                std::thread::sleep(std::time::Duration::from_secs(2));
            }

            #[cfg(windows)]
            {
                Command::new("taskkill")
                    .arg("/PID")
                    .arg(&pid_str)
                    .arg("/F")
                    .output()
                    .context("Failed to stop SurrealDB process")?;
            }

            let _ = fs::remove_file(&pid_path);
        }

        Ok(())
    }

    pub fn start_service_for_database(db_info: &DatabaseInfo) -> Result<()> {
        // Qdrant 使用直接二进制进程管理
        if db_info.db_type == DatabaseType::Qdrant {
            let binary_path = Path::new(&db_info.install_path).join("qdrant");
            let data_dir = Path::new(&db_info.data_path);
            let config_path = db_info
                .config
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    data_dir
                        .parent()
                        .unwrap()
                        .join("config")
                        .join("qdrant")
                        .join("config.yaml")
                });
            return start_qdrant_process(&binary_path, &config_path, data_dir);
        }

        // SurrealDB 使用直接二进制进程管理
        if db_info.db_type == DatabaseType::SurrealDB {
            return start_surrealdb_process(db_info);
        }

        let brew = Homebrew::bootstrap()?;
        let recipe = HomebrewDatabaseRecipe::resolve(&db_info.db_type)?;
        brew.start_service_with_retry(recipe.service_name)
    }

    pub fn stop_service_for_database(db_info: &DatabaseInfo) -> Result<()> {
        // Qdrant 使用直接二进制进程管理
        if db_info.db_type == DatabaseType::Qdrant {
            return stop_qdrant_process(db_info);
        }

        // SurrealDB 使用直接二进制进程管理
        if db_info.db_type == DatabaseType::SurrealDB {
            return stop_surrealdb_process(db_info);
        }

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
            DatabaseType::Qdrant => configure_qdrant(brew, storage_path, port),
            DatabaseType::SurrealDB => configure_surrealdb(brew, storage_path, port),
            DatabaseType::Neo4j | DatabaseType::SeekDB => {
                bail!("Configuration for {:?} not implemented", db_type)
            }
        }
    }

    fn configure_redis(brew: &Homebrew, storage_path: &Path) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("redis"))?;
        let etc_dir = prefix.join("etc");
        utils::ensure_dir(&etc_dir)?;
        let conf_path = etc_dir.join("redis.conf");

        let data_dir = utils::get_db_data_path(storage_path, "redis");
        let logs_dir = utils::get_db_log_path(storage_path, "redis");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("redis.log");

        // 如果配置文件存在，读取并更新；否则创建新配置
        let contents = if conf_path.exists() {
            fs::read_to_string(&conf_path)
                .with_context(|| format!("Failed to read redis.conf at {}", conf_path.display()))?
        } else {
            String::new()
        };

        let mut rewritten = String::with_capacity(contents.len() + 200);
        let mut has_dir = false;
        let mut has_logfile = false;

        for line in contents.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("dir ") {
                rewritten.push_str(&format!("dir {}\n", data_dir.display()));
                has_dir = true;
            } else if trimmed.starts_with("logfile ") {
                rewritten.push_str(&format!("logfile {}\n", log_file.display()));
                has_logfile = true;
            } else {
                rewritten.push_str(line);
                rewritten.push('\n');
            }
        }

        // 如果配置文件是新的或者缺少必要项，追加默认配置
        if contents.is_empty() || !has_dir {
            rewritten.push_str(&format!("dir {}\n", data_dir.display()));
        }
        if contents.is_empty() || !has_logfile {
            rewritten.push_str(&format!("logfile {}\n", log_file.display()));
        }
        // 如果是空配置文件，添加一些基本设置
        if contents.is_empty() {
            rewritten.push_str("appendonly no\n");
            rewritten.push_str("loglevel notice\n");
        }

        fs::write(&conf_path, rewritten).with_context(|| "Failed to write redis.conf")?;

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn configure_mysql(brew: &Homebrew, storage_path: &Path, port: u16) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("mysql@8.4"))?;
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
        port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("postgresql@18"))?;
        let etc_dir = prefix.join("etc");
        utils::ensure_dir(&etc_dir)?;
        let conf_path = etc_dir.join("postgresql.conf");

        let data_dir = utils::get_db_data_path(storage_path, "postgresql");
        let logs_dir = utils::get_db_log_path(storage_path, "postgresql");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("postgresql.log");

        let user = std::env::var("USER").unwrap_or_else(|_| "local".to_string());

        let config_content = format!(
            "# PostgreSQL configuration for local-db\n\
            # Port configuration\n\
            port = {}\n\
            \n\
            # Connection settings\n\
            listen_addresses = 'localhost'\n\
            max_connections = 100\n\
            \n\
            # Memory settings\n\
            shared_buffers = 128MB\n\
            effective_cache_size = 512MB\n\
            maintenance_work_mem = 32MB\n\
            checkpoint_completion_target = 0.9\n\
            wal_buffers = 4MB\n\
            default_statistics_target = 100\n\
            random_page_cost = 1.1\n\
            effective_io_concurrency = 200\n\
            work_mem = 2621kB\n\
            min_wal_size = 1GB\n\
            max_wal_size = 4GB\n\
            \n\
            # Logging\n\
            logging_collector = on\n\
            log_directory = '{}'\n\
            log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'\n\
            log_rotation_age = 1d\n\
            log_rotation_size = 100MB\n\
            log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '\n\
            log_timezone = 'UTC'\n\
            \n\
            # Locale and encoding\n\
            datestyle = 'iso, mdy'\n\
            timezone = 'UTC'\n\
            default_text_search_config = 'pg_catalog.english'\n",
            port,
            logs_dir.display()
        );
        fs::write(&conf_path, config_content).with_context(|| "Failed to write postgresql.conf")?;

        initialize_postgresql_data_dir(&prefix, &data_dir, &user)?;

        Ok(ConfiguredPaths {
            config_path: conf_path,
            log_path: log_file,
        })
    }

    fn initialize_postgresql_data_dir(prefix: &Path, data_dir: &Path, user: &str) -> Result<()> {
        let mut entries = fs::read_dir(data_dir)?;
        if entries.next().is_some() {
            return Ok(());
        }

        let initdb_path = prefix.join("bin").join("initdb");
        if !initdb_path.exists() {
            bail!("initdb binary not found at {}", initdb_path.display());
        }

        let output = Command::new(&initdb_path)
            .arg("-D")
            .arg(data_dir)
            .arg(format!("--username={}", user))
            .arg("--auth-local=trust")
            .arg("--auth-host=trust")
            .arg("--encoding=UTF8")
            .arg("--no-locale")
            .output()
            .with_context(|| "Failed to initialize PostgreSQL data directory")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            bail!(
                "initdb failed: {}\nstdout: {}\nstderr: {}",
                output.status,
                stdout,
                stderr
            );
        }

        // Create admin user with default password after initialization
        // Start PostgreSQL temporarily to create the user
        let pg_ctl_path = prefix.join("bin").join("pg_ctl");
        let psql_path = prefix.join("bin").join("psql");

        if pg_ctl_path.exists() && psql_path.exists() {
            // Start PostgreSQL temporarily
            let _ = Command::new(&pg_ctl_path)
                .arg("-D")
                .arg(data_dir)
                .arg("-l")
                .arg(data_dir.join("logfile"))
                .arg("start")
                .arg("-w")
                .status();

            // Wait for PostgreSQL to be ready
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Create admin user with password admin888
            let _ = Command::new(&psql_path)
                .arg("-U")
                .arg(user)
                .arg("-d")
                .arg("postgres")
                .arg("-c")
                .arg("CREATE ROLE admin WITH LOGIN SUPERUSER CREATEDB CREATEROLE PASSWORD 'admin888';")
                .status();

            // Stop PostgreSQL (it will be started by brew services later)
            let _ = Command::new(&pg_ctl_path)
                .arg("-D")
                .arg(data_dir)
                .arg("stop")
                .arg("-m")
                .arg("fast")
                .status();
        }

        Ok(())
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

    fn configure_surrealdb(
        brew: &Homebrew,
        storage_path: &Path,
        port: u16,
    ) -> Result<ConfiguredPaths> {
        let prefix = brew.prefix(Some("surreal"))?;
        let etc_dir = prefix.join("etc");
        utils::ensure_dir(&etc_dir)?;
        let conf_path = etc_dir.join("surrealdb.env");

        let data_dir = utils::get_db_data_path(storage_path, "surrealdb");
        let logs_dir = utils::get_db_log_path(storage_path, "surrealdb");
        utils::ensure_dir(&data_dir)?;
        utils::ensure_dir(&logs_dir)?;
        let log_file = logs_dir.join("surrealdb.log");

        // SurrealDB 配置 - 创建一个环境变量配置文件用于 launchd
        let config_content = format!(
            "# SurrealDB configuration for local-db\n\
            # Data directory\n\
            SURREAL_PATH={}\n\
            # Server bind address\n\
            SURREAL_BIND=0.0.0.0:{}\n\
            # Log level\n\
            SURREAL_LOG=info\n\
            # Log file\n\
            SURREAL_LOG_FILE={}\n",
            data_dir.display(),
            port,
            log_file.display()
        );
        fs::write(&conf_path, config_content).with_context(|| "Failed to write surrealdb.env")?;

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
            DatabaseType::Redis | DatabaseType::Qdrant => None,
            _ => Some("admin".to_string()),
        }
    }

    fn resolve_password(db_type: &DatabaseType, override_password: Option<&str>) -> Option<String> {
        if let Some(pwd) = override_password {
            return Some(pwd.to_string());
        }
        match db_type {
            DatabaseType::Redis | DatabaseType::Qdrant => None,
            _ => Some("admin888".to_string()),
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

        fn start_service_with_retry(&self, service: &str) -> Result<()> {
            match self.start_service(service) {
                Ok(_) => Ok(()),
                Err(e) => {
                    let err_msg = e.to_string();
                    // 检查是否是 Bootstrap failed 错误
                    if err_msg.contains("Bootstrap failed: 5")
                        || err_msg.contains("Input/output error")
                    {
                        eprintln!(
                            "Start service failed with bootstrap error, attempting recovery for {}",
                            service
                        );

                        // 尝试停止服务以清理状态
                        let _ = self.stop_service(service);

                        // 等待一小会儿让 launchd 清理
                        std::thread::sleep(std::time::Duration::from_secs(2));

                        // 再次尝试启动
                        eprintln!("Retrying start service for {}", service);
                        self.start_service(service)
                    } else {
                        Err(e)
                    }
                }
            }
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

        fn get_service_status(&self, service: &str) -> Result<bool> {
            let output = Command::new(&self.bin_path)
                .args(["services", "list"])
                .output()
                .with_context(|| "Failed to list brew services")?;
            if !output.status.success() {
                bail!("Failed to list brew services");
            }
            let output_text = String::from_utf8_lossy(&output.stdout);
            Ok(output_text.contains(service) && output_text.contains("started"))
        }

        fn list_services(&self) -> Result<String> {
            let output = Command::new(&self.bin_path)
                .args(["services", "list"])
                .output()
                .with_context(|| "Failed to list brew services")?;
            if !output.status.success() {
                bail!("Failed to list brew services");
            }
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }

    struct HomebrewDatabaseRecipe {
        #[allow(dead_code)]
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
                    formula: "mysql@8.4",
                    service_name: "mysql@8.4",
                    port: 3306,
                }),
                DatabaseType::PostgreSQL => Ok(Self {
                    db_type: DatabaseType::PostgreSQL,
                    tap: None,
                    formula: "postgresql@18",
                    service_name: "postgresql@18",
                    port: 5432,
                }),
                DatabaseType::MongoDB => Ok(Self {
                    db_type: DatabaseType::MongoDB,
                    tap: Some("mongodb/brew"),
                    formula: "mongodb-community@7.0",
                    service_name: "mongodb-community@7.0",
                    port: 27017,
                }),
                DatabaseType::Qdrant => {
                    bail!("Qdrant should be installed via binary, not Homebrew")
                }
                DatabaseType::SurrealDB => {
                    bail!("SurrealDB should be started via direct process, not brew services")
                }
                DatabaseType::Neo4j => bail!("Neo4j not yet implemented on macOS"),
                DatabaseType::SeekDB => bail!("SeekDB not yet implemented on macOS"),
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

    pub fn get_installed_databases_from_homebrew() -> Vec<DatabaseType> {
        Vec::new()
    }

    pub fn get_homebrew_service_status(_service: &str) -> Option<bool> {
        None
    }

    pub fn get_all_homebrew_services_status() -> std::collections::HashMap<String, bool> {
        std::collections::HashMap::new()
    }
}

// macOS 导出
#[cfg(target_os = "macos")]
pub use imp::{
    get_all_homebrew_services_status, install_database_via_homebrew, start_service_for_database,
    stop_service_for_database,
};

// 非 macOS 导出
#[cfg(not(target_os = "macos"))]
pub use imp::{
    get_all_homebrew_services_status, install_database_via_homebrew, start_service_for_database,
    stop_service_for_database,
};
