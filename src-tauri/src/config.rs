use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

const AUTOSTART_VALUE_NAME: &str = "TakeABreakPortable";

/// 应用持久化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub silent_start: bool,
    pub auto_rest_secs: u64,
    pub sitting_interval_secs: u64,
    pub water_interval_secs: u64,
    pub rest_duration_secs: u64,
    pub extend_duration_secs: u64,
    pub rest_enabled: bool,
    pub water_enabled: bool,
    pub autostart_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            silent_start: false,
            auto_rest_secs: 10,
            sitting_interval_secs: 50 * 60,
            water_interval_secs: 80 * 60,
            rest_duration_secs: 5 * 60,
            extend_duration_secs: 5 * 60,
            rest_enabled: true,
            water_enabled: true,
            autostart_enabled: false,
        }
    }
}

/// 前端设置页初始化数据
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPayload {
    pub config: AppConfig,
    pub config_path: String,
    pub autostart_supported: bool,
}

/// 便携版配置统一放在程序目录下的 data/config.json
pub fn config_path() -> PathBuf {
    let base_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    base_dir.join("data").join("config.json")
}

/// 确保配置目录存在，便于前端直接打开文件夹
fn ensure_config_dir() {
    if let Some(parent) = config_path().parent() {
        let _ = fs::create_dir_all(parent);
    }
}

/// 读取本地配置文件；缺失或损坏时回退到默认配置
pub fn load_config() -> AppConfig {
    ensure_config_dir();
    let path = config_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<AppConfig>(&raw).ok())
        .unwrap_or_default()
}

/// 保存本地配置文件
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let raw = serde_json::to_string_pretty(config).map_err(|err| err.to_string())?;
    fs::write(path, raw).map_err(|err| err.to_string())
}

/// 当前平台是否支持本实现的开机启动
pub fn autostart_supported() -> bool {
    cfg!(target_os = "windows")
}

/// 查询开机启动状态
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let exe_path = match std::env::current_exe() {
            Ok(path) => format!("\"{}\"", path.display()),
            Err(_) => return false,
        };
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = match hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
            Ok(key) => key,
            Err(_) => return false,
        };

        return run_key
            .get_value::<String, _>(AUTOSTART_VALUE_NAME)
            .map(|value| value == exe_path)
            .unwrap_or(false);
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// 切换开机启动状态
pub fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run_key, _) = hkcu
            .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
            .map_err(|err| err.to_string())?;

        if enabled {
            let exe_path = std::env::current_exe().map_err(|err| err.to_string())?;
            let quoted_path = format!("\"{}\"", exe_path.display());
            run_key
                .set_value(AUTOSTART_VALUE_NAME, &quoted_path)
                .map_err(|err| err.to_string())?;
            return Ok(());
        }

        if !is_autostart_enabled() {
            return Ok(());
        }

        run_key
            .delete_value(AUTOSTART_VALUE_NAME)
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
        Err("当前平台暂不支持开机启动".to_string())
    }
}
