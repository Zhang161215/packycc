use super::types::Config;
use std::path::{Path, PathBuf};
use std::fs;

pub struct ConfigLoader;

impl ConfigLoader {
    fn get_config_path() -> PathBuf {
        // 获取当前exe文件所在目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join("config.toml");
            }
        }
        // 如果获取exe路径失败，使用当前工作目录
        PathBuf::from("config.toml")
    }

    pub fn load() -> Config {
        let config_path = Self::get_config_path();
        
        if config_path.exists() {
            match Self::load_from_path(&config_path) {
                Ok(mut config) => {
                    // 如果配置文件存在，说明不是首次运行
                    config.first_run = false;
                    config
                }
                Err(_e) => {
                    Config::default()
                }
            }
        } else {
            // 配置文件不存在，创建默认配置并保存
            let mut config = Config::default();
            config.first_run = true;
            
            // 创建配置目录
            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            
            // 保存配置文件，标记为非首次运行
            let mut save_config = config.clone();
            save_config.first_run = false;
            let _ = Self::save_config(&save_config, &config_path);
            
            config
        }
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_config(config: &Config, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }
}
