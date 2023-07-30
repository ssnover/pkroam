use std::path::PathBuf;
use std::str::FromStr;

use super::Cli;

pub struct AppPaths {
    config_dir: PathBuf,
}

impl AppPaths {
    pub fn from_dir(dir: PathBuf) -> Self {
        AppPaths { config_dir: dir }
    }

    pub fn get_database_path(&self) -> PathBuf {
        let mut database_path = self.config_dir.clone();
        database_path.push("db");
        let _ = std::fs::create_dir_all(&database_path);
        database_path.push("pkroam.sqlite");
        database_path
    }

    pub fn get_backup_path(&self) -> PathBuf {
        let mut backup_path = self.config_dir.clone();
        backup_path.push(".backups");
        let _ = std::fs::create_dir_all(&backup_path);
        backup_path
    }

    pub fn get_log_path(&self) -> PathBuf {
        let mut log_path = self.config_dir.clone();
        log_path.push("logs");
        let _ = std::fs::create_dir_all(&log_path);
        log_path
    }
}

pub fn get_app_paths(args: &Cli) -> Result<AppPaths, String> {
    let config_dir = get_config_dir(args)?;
    let _ = std::fs::create_dir_all(&config_dir);
    Ok(AppPaths::from_dir(config_dir))
}

fn get_config_dir(args: &Cli) -> Result<PathBuf, String> {
    if let Some(config_dir) = args.config_dir.clone() {
        Ok(config_dir)
    } else if let Ok(Ok(env_config_dir)) =
        std::env::var("PKROAM_CONFIG_DIR").map(|path_str| PathBuf::from_str(&path_str))
    {
        Ok(env_config_dir)
    } else if let Some(base_dirs) = directories::BaseDirs::new() {
        let mut config_dir = base_dirs.data_local_dir().to_path_buf();
        config_dir.push("pkroam");
        Ok(config_dir)
    } else {
        Err(String::from("No suitable configuration directory found"))
    }
}
