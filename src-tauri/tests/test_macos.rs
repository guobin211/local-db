#![cfg(target_os = "macos")]

use anyhow::{Context, Result};
use std::process::Command;

const HOMEBREW_INSTALL_SCRIPT: &str =
    "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh";

#[derive(Clone, Copy)]
struct BrewDatabase {
    tap: Option<&'static str>,
    formula: &'static str,
    service: &'static str,
}

const MONGODB: BrewDatabase = BrewDatabase {
    tap: Some("mongodb/brew"),
    formula: "mongodb-community@7.0",
    service: "mongodb-community@7.0",
};

const MYSQL: BrewDatabase = BrewDatabase {
    tap: None,
    formula: "mysql@8.4",
    service: "mysql@8.4",
};

const REDIS: BrewDatabase = BrewDatabase {
    tap: None,
    formula: "redis",
    service: "redis",
};

const POSTGRESQL: BrewDatabase = BrewDatabase {
    tap: None,
    formula: "postgresql@18",
    service: "postgresql@18",
};

const QDRANT: BrewDatabase = BrewDatabase {
    tap: None,
    formula: "qdrant",
    service: "qdrant",
};

const SURREALDB: BrewDatabase = BrewDatabase {
    tap: Some("surrealdb/tap"),
    formula: "surreal",
    service: "surreal",
};

fn run_shell(command: &str) -> Result<()> {
    let status = Command::new("/bin/bash")
        .arg("-c")
        .arg(command)
        .status()
        .with_context(|| format!("Failed to execute shell command: {}", command))?;

    if !status.success() {
        anyhow::bail!("Command `{}` exited with status {}", command, status);
    }

    Ok(())
}

fn brew_exists() -> bool {
    Command::new("/usr/bin/env")
        .arg("which")
        .arg("brew")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn ensure_homebrew_installed() -> Result<()> {
    if brew_exists() {
        return Ok(());
    }

    let install_cmd = format!(
        "NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL {})\"",
        HOMEBREW_INSTALL_SCRIPT
    );
    run_shell(&install_cmd)
}

fn run_brew(args: &[&str]) -> Result<String> {
    ensure_homebrew_installed()?;

    let output = Command::new("brew")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run brew {}", args.join(" ")))?;

    if !output.status.success() {
        anyhow::bail!(
            "brew {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn ensure_tap(tap: &str) -> Result<()> {
    run_brew(&["tap", tap]).map(|_| ())
}

fn is_formula_installed(formula: &str) -> Result<bool> {
    let output = run_brew(&["list", "--formula", "--versions", formula])?;
    Ok(!output.trim().is_empty())
}

fn ensure_database_installed(db: BrewDatabase) -> Result<()> {
    if let Some(tap) = db.tap {
        ensure_tap(tap)?;
    }

    if !is_formula_installed(db.formula)? {
        run_brew(&["install", db.formula])?;
    }

    Ok(())
}

fn ensure_database_service_running(db: BrewDatabase) -> Result<()> {
    ensure_database_installed(db)?;
    run_brew(&["services", "start", db.service]).map(|_| ())
}

macro_rules! define_install_test {
    ($fn_name:ident, $db:expr, $desc:literal) => {
        #[test]
        #[ignore = $desc]
        fn $fn_name() -> Result<()> {
            ensure_database_installed($db)
        }
    };
}

macro_rules! define_service_test {
    ($fn_name:ident, $db:expr, $desc:literal) => {
        #[test]
        #[ignore = $desc]
        fn $fn_name() -> Result<()> {
            ensure_database_service_running($db)
        }
    };
}

#[test]
#[ignore = "Requires macOS environment with network access"]
fn test_install_homebrew() -> Result<()> {
    ensure_homebrew_installed()
}

define_install_test!(
    test_install_mongodb_via_homebrew,
    MONGODB,
    "Requires MongoDB tap and formula access"
);
define_install_test!(
    test_install_mysql_via_homebrew,
    MYSQL,
    "Requires Homebrew formula mysql"
);
define_install_test!(
    test_install_redis_via_homebrew,
    REDIS,
    "Requires Homebrew formula redis"
);

define_install_test!(
    test_install_postgresql_via_homebrew,
    POSTGRESQL,
    "Requires Homebrew formula postgresql@18"
);

define_install_test!(
    test_install_qdrant_via_homebrew,
    QDRANT,
    "Requires Homebrew formula qdrant"
);
define_install_test!(
    test_install_surrealdb_via_homebrew,
    SURREALDB,
    "Requires SurrealDB Homebrew tap"
);

define_service_test!(
    test_start_mongodb_via_homebrew,
    MONGODB,
    "Requires MongoDB Homebrew service support"
);
define_service_test!(
    test_start_mysql_via_homebrew,
    MYSQL,
    "Requires MySQL Homebrew service support"
);
define_service_test!(
    test_start_redis_via_homebrew,
    REDIS,
    "Requires Redis Homebrew service support"
);
define_service_test!(
    test_start_postgresql_via_homebrew,
    POSTGRESQL,
    "Requires PostgreSQL Homebrew service support"
);
define_service_test!(
    test_start_qdrant_via_homebrew,
    QDRANT,
    "Requires Qdrant Homebrew service support"
);
define_service_test!(
    test_start_surrealdb_via_homebrew,
    SURREALDB,
    "Requires SurrealDB Homebrew service support"
);
