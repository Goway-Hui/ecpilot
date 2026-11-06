//! 热保护参数：GUI（反序列化显示）与 root helper（执行）共用一份定义。
//!
//! 两端必须保持同一份结构；此前 `ec.rs` 与 `thermal.rs` 各有一份，
//! helper 增加字段或修改默认值时 GUI 会静默落回默认值。

/// 温控参数；可通过 `MCC_THERMAL_*` 环境变量覆盖。
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ThermalConfig {
    pub up1: i32,
    pub up2: i32,
    pub up3: i32,
    pub down1: i32,
    pub down2: i32,
    pub down3: i32,
    pub cap0: i32,
    pub cap1: i32,
    pub cap2: i32,
    pub cap3: i32,
    pub base_khz: i32,
    pub dwell_ms: u64,
    pub sample_ms: u64,
    pub latch_temp: i32,
    pub latch_secs: u64,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            up1: 90,
            up2: 97,
            up3: 101,
            down1: 85,
            down2: 92,
            down3: 96,
            cap0: 4_200_000,
            cap1: 3_800_000,
            cap2: 2_400_000,
            cap3: 1_500_000,
            base_khz: 2_501_000,
            dwell_ms: 8_000,
            sample_ms: 200,
            latch_temp: 90,
            latch_secs: 30,
        }
    }
}

impl ThermalConfig {
    /// 读取 `MCC_THERMAL_*` 覆盖值；helper 启动时调用一次。
    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.up1 = env_i32("MCC_THERMAL_UP1", config.up1);
        config.up2 = env_i32("MCC_THERMAL_UP2", config.up2);
        config.up3 = env_i32("MCC_THERMAL_UP3", config.up3);
        config.down1 = env_i32("MCC_THERMAL_DOWN1", config.down1);
        config.down2 = env_i32("MCC_THERMAL_DOWN2", config.down2);
        config.down3 = env_i32("MCC_THERMAL_DOWN3", config.down3);
        config.cap0 = env_i32("MCC_THERMAL_CAP0", config.cap0);
        config.cap1 = env_i32("MCC_THERMAL_CAP1", config.cap1);
        config.cap2 = env_i32("MCC_THERMAL_CAP2", config.cap2);
        config.cap3 = env_i32("MCC_THERMAL_CAP3", config.cap3);
        config.base_khz = env_i32("MCC_THERMAL_BASE_KHZ", config.base_khz);
        config.dwell_ms = env_u64("MCC_THERMAL_DWELL_MS", config.dwell_ms).max(1_000);
        config.sample_ms = env_u64("MCC_THERMAL_SAMPLE_MS", config.sample_ms).clamp(50, 2_000);
        config.latch_temp = env_i32("MCC_THERMAL_LATCH_TEMP", config.latch_temp);
        config.latch_secs = env_u64("MCC_THERMAL_LATCH_SECS", config.latch_secs).max(1);
        config
    }
}

fn env_i32(key: &str, default: i32) -> i32 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
