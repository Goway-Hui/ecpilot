//! 语言、主题、自定义风扇曲线的配置读写。
//!
//! 配置文件位于 `$XDG_CONFIG_HOME/ecpilot/`，读取失败时使用默认值。

use crate::theme::ThemeMode;

#[derive(Clone, Copy, PartialEq)]
/// 界面语言。
pub(crate) enum Lang {
    Zh,
    En,
}

#[derive(Clone, Copy, PartialEq)]
/// 自定义曲线当前编辑的目标（CPU 或 GPU）。
pub(crate) enum CurveTarget {
    Cpu,
    Gpu,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
/// 用户自定义风扇曲线：6 个温度点 + 7 个转速点。
pub(crate) struct CustomCurve {
    pub(crate) cpu_temp: [u8; 6],
    pub(crate) cpu_speed: [u8; 7],
    pub(crate) gpu_temp: [u8; 6],
    pub(crate) gpu_speed: [u8; 7],
}

impl Default for CustomCurve {
    fn default() -> Self {
        Self {
            cpu_temp: [50, 57, 65, 75, 85, 95],
            cpu_speed: [0, 40, 60, 91, 125, 140, 150],
            gpu_temp: [55, 60, 65, 70, 75, 80],
            gpu_speed: [0, 48, 55, 62, 68, 77, 85],
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
/// 持久化配置（语言、主题）。
pub(crate) struct Config {
    /// 配置结构版本，便于以后迁移。
    pub(crate) version: Option<u32>,
    pub(crate) theme: Option<String>,
    pub(crate) lang: Option<String>,
    /// 关闭窗口时是否最小化到托盘；缺省开启。
    pub(crate) close_to_tray: Option<bool>,
    /// 上次使用的风扇模式（auto/silent/advanced）。
    pub(crate) fan_mode: Option<String>,
    /// 上次使用的曲线预设（default/quiet/gaming/max/custom）。
    pub(crate) fan_preset: Option<String>,
    /// 上次使用的性能模式（eco/comfort/turbo）。
    pub(crate) shift_mode: Option<String>,
}

fn config_base() -> std::path::PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn config_path() -> std::path::PathBuf {
    config_base().join("ecpilot").join("config.json")
}

/// 从旧 `mcontrolcenter-gpui` 配置目录迁移一次；不覆盖已有 ECPilot 配置。
pub(crate) fn migrate_legacy_configs() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let base = config_base();
        let old = base.join("mcontrolcenter-gpui");
        let new = base.join("ecpilot");
        if !old.is_dir() || new.join("config.json").exists() {
            return;
        }
        let _ = std::fs::create_dir_all(&new);
        for name in ["config.json", "custom.json"] {
            let source = old.join(name);
            if source.is_file() {
                let _ = std::fs::copy(&source, new.join(name));
            }
        }
    });
}

/// 读取配置；文件缺失或损坏时返回默认值。
pub(crate) fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// 读取风扇配置：模式、曲线预设、性能模式。
pub(crate) fn load_fan_settings() -> (Option<String>, Option<String>, Option<String>) {
    let config = load_config();
    (config.fan_mode, config.fan_preset, config.shift_mode)
}

/// 保存风扇模式。
pub(crate) fn save_fan_mode(mode: &str) {
    let mut config = load_config();
    config.fan_mode = Some(mode.to_string());
    write_config(&config);
}

/// 保存曲线预设（custom 表示自定义曲线）。
pub(crate) fn save_fan_preset(preset: &str) {
    let mut config = load_config();
    config.fan_preset = Some(preset.to_string());
    write_config(&config);
}

/// 保存性能模式。
pub(crate) fn save_shift_mode(mode: &str) {
    let mut config = load_config();
    config.shift_mode = Some(mode.to_string());
    write_config(&config);
}

/// 保存语言与主题设置。
pub(crate) fn save_config(theme: ThemeMode, lang: Lang) {
    let mut config = load_config();
    config.theme = Some(
        match theme {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        }
        .to_string(),
    );
    config.lang = Some(
        match lang {
            Lang::Zh => "zh",
            Lang::En => "en",
        }
        .to_string(),
    );
    write_config(&config);
}

fn write_config(config: &Config) {
    let mut config = config.clone();
    config.version = Some(1);
    let config = &config;
    if let Some(parent) = config_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(config) {
        let temp = config_path().with_extension("json.tmp");
        if std::fs::write(&temp, text).is_ok() {
            let _ = std::fs::rename(&temp, config_path());
        }
    }
}

/// 关闭窗口时是否最小化到托盘，缺省开启（niri 关闭快捷键不会结束进程）。
pub(crate) fn load_close_to_tray() -> bool {
    load_config().close_to_tray.unwrap_or(true)
}

/// 保存“关闭到托盘”偏好。
pub(crate) fn save_close_to_tray(enabled: bool) {
    let mut config = load_config();
    config.close_to_tray = Some(enabled);
    write_config(&config);
}

fn custom_path() -> std::path::PathBuf {
    config_path().with_file_name("custom.json")
}

/// 读取自定义风扇曲线；自动清理越界/非单调数据。
pub(crate) fn load_custom() -> CustomCurve {
    let mut curve: CustomCurve = std::fs::read_to_string(custom_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    curve.sanitize();
    curve
}

impl CustomCurve {
    fn sanitize(&mut self) {
        let clamp_temp = |temps: &mut [u8; 6]| {
            let mut previous: i32 = 40;
            for temp in temps.iter_mut() {
                let value = (*temp as i32).clamp(previous, 95);
                *temp = value as u8;
                previous = value;
            }
        };
        clamp_temp(&mut self.cpu_temp);
        clamp_temp(&mut self.gpu_temp);
        let clamp_speed = |speeds: &mut [u8; 7]| {
            let mut previous: i32 = 0;
            for speed in speeds.iter_mut() {
                let value = (*speed as i32).clamp(previous, 150);
                *speed = value as u8;
                previous = value;
            }
            if speeds[6] < 70 {
                speeds[6] = 70;
            }
        };
        clamp_speed(&mut self.cpu_speed);
        clamp_speed(&mut self.gpu_speed);
    }
}

/// 保存自定义风扇曲线。
pub(crate) fn save_custom(curve: &CustomCurve) {
    if let Some(parent) = custom_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(curve) {
        let _ = std::fs::write(custom_path(), text);
    }
}

