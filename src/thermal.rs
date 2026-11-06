//! 内置多级温度保护。
//!
//! 由 root helper 常驻运行：每 200 ms 读取 k10temp 的温度，按档位写
//! cpufreq / EPP / boost。与旧脚本的区别：
//! - 同时参考 Tctl、Tccd1、Tccd2 的最高值；
//! - 写入后读回校验，错误通过 `thermalStatus` 返回；
//! - 启用状态持久化到 `/var/lib/ecpilot/thermal-enabled`；
//! - helper 收到 SIGTERM / 线程 panic 时落安全档；
//! - 阈值和 caps 可由 `MCC_THERMAL_*` 环境变量覆盖。

use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CPUFREQ: &str = "/sys/devices/system/cpu/cpufreq";
const ENABLED_PATH: &str = "/var/lib/ecpilot/thermal-enabled";
const LEGACY_ENABLED_PATH: &str = "/var/lib/mcontrolcenter-gpui/thermal-enabled";
const MANUAL_PATH: &str = "/var/lib/ecpilot/thermal-manual";
const CAP_PATH: &str = "/var/lib/ecpilot/thermal-cap-khz";
const BOOST_PATH: &str = "/var/lib/ecpilot/thermal-boost";
const LEVEL_PATH: &str = "/var/lib/ecpilot/thermal-level";
const MIN_CAP_KHZ: i32 = 800_000;

use crate::thermal_config::ThermalConfig;

/// 手动模式的目标值；整体加锁，避免 worker 读到半更新的组合。
#[derive(Clone, Copy)]
struct ManualState {
    level: i32,
    cap: i32,
    boost: bool,
}

/// helper 与后台线程共享的温控状态。
pub struct ThermalGuard {
    config: ThermalConfig,
    enabled: AtomicBool,
    level: AtomicI32,
    cap_khz: AtomicI32,
    temp: AtomicI32,
    boost: AtomicBool,
    manual: AtomicBool,
    manual_state: Mutex<ManualState>,
    manual_dirty: AtomicBool,
    restart: AtomicBool,
    no_boost: bool,
    error: Mutex<Option<String>>,
}

impl ThermalGuard {
    /// 启动温控线程；启用状态从磁盘恢复。
    pub fn start() -> Arc<Self> {
        let config = ThermalConfig::from_env();
        let enabled = load_enabled();
        let no_boost = last_boot_thermal_trip();
        let manual = load_bool(MANUAL_PATH, false);
        let manual_level = load_i32(LEVEL_PATH, 0).clamp(-1, 3);
        let manual_cap =
            load_i32(CAP_PATH, config.cap0).clamp(MIN_CAP_KHZ, max_cap_khz());
        let manual_boost = load_bool(BOOST_PATH, true) && !no_boost;
        let level = if manual { manual_level } else { 0 };
        let cap = if enabled {
            if manual { manual_cap } else { config.cap0 }
        } else {
            config.base_khz
        };
        let boost = enabled && if manual { manual_boost } else { !no_boost };
        let guard = Arc::new(Self {
            config,
            enabled: AtomicBool::new(enabled),
            level: AtomicI32::new(level),
            cap_khz: AtomicI32::new(cap),
            temp: AtomicI32::new(0),
            boost: AtomicBool::new(boost),
            manual: AtomicBool::new(manual),
            manual_state: Mutex::new(ManualState {
                level: manual_level,
                cap: manual_cap,
                boost: manual_boost,
            }),
            manual_dirty: AtomicBool::new(false),
            restart: AtomicBool::new(false),
            no_boost,
            error: Mutex::new(None),
        });
        let worker = Arc::clone(&guard);
        std::thread::Builder::new()
            .name("thermal-guard".into())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| run(&worker)));
                if result.is_err() {
                    worker.shutdown_safe();
                    worker.set_error("thermal guard thread panicked");
                }
            })
            .ok();
        guard
    }

    /// 返回 `(enabled, level, cap_khz, temp_max, boost, error)`。
    pub fn status(&self) -> (bool, i32, i32, i32, bool, String) {
        (
            self.enabled.load(Ordering::Relaxed),
            self.level.load(Ordering::Relaxed),
            self.cap_khz.load(Ordering::Relaxed),
            self.temp.load(Ordering::Relaxed),
            self.boost.load(Ordering::Relaxed),
            self.error
                .lock()
                .ok()
                .and_then(|error| error.clone())
                .unwrap_or_default(),
        )
    }

    /// 参数 JSON，供 GUI 显示实际阈值。
    pub fn config_json(&self) -> String {
        serde_json::to_string(&self.config).unwrap_or_default()
    }

    /// 是否处于手动模式。
    pub fn manual(&self) -> bool {
        self.manual.load(Ordering::SeqCst)
    }

    /// 启用/停用并持久化；停用时线程落安全档。
    pub fn set_enabled(&self, on: bool) -> Result<(), String> {
        // 先写磁盘，磁盘失败时不要把内存状态也翻转，避免状态与磁盘不一致。
        write_state(ENABLED_PATH, if on { "1" } else { "0" })?;
        self.enabled.store(on, Ordering::Relaxed);
        if on {
            self.manual_dirty.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    /// 切换自动/手动模式并持久化。
    pub fn set_manual(&self, manual: bool) -> Result<(), String> {
        write_state(MANUAL_PATH, if manual { "1" } else { "0" })?;
        self.manual.store(manual, Ordering::SeqCst);
        if manual {
            self.manual_dirty.store(true, Ordering::SeqCst);
        } else {
            self.restart.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    /// 手动选择 0..=3 档，使用该档的配置频率与 boost 策略。
    pub fn set_level(&self, level: i32) -> Result<(), String> {
        if !(0..=3).contains(&level) {
            return Err("thermal level must be 0..=3".into());
        }
        let (cap, boost) = level_target(&self.config, level, self.no_boost);
        self.store_manual(level, cap, boost)
    }

    /// 手动设置频率上限（kHz）与 boost。
    pub fn set_frequency(&self, cap_khz: i32, boost: bool) -> Result<(), String> {
        if boost && self.no_boost {
            return Err("boost is disabled after a thermal trip".into());
        }
        let mut cap = cap_khz.clamp(MIN_CAP_KHZ, max_cap_khz());
        // 关闭 boost 时内核只能到基础频率；把上限钳到基础频率，
        // 避免写入后被内核拒绝。提高上限时由 GUI 自动开启 boost。
        if !boost && cap > self.config.base_khz {
            cap = self.config.base_khz;
        }
        self.store_manual(-1, cap, boost)
    }

    fn store_manual(&self, level: i32, cap: i32, boost: bool) -> Result<(), String> {
        write_state(MANUAL_PATH, "1")?;
        write_state(CAP_PATH, &cap.to_string())?;
        write_state(BOOST_PATH, if boost { "1" } else { "0" })?;
        write_state(LEVEL_PATH, &level.to_string())?;
        if let Ok(mut state) = self.manual_state.lock() {
            *state = ManualState { level, cap, boost };
        }
        self.manual.store(true, Ordering::SeqCst);
        self.manual_dirty.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// 重新应用参数：自动模式回到 0 档，手动模式重放当前设定。
    pub fn restart(&self) {
        self.restart.store(true, Ordering::Relaxed);
        self.manual_dirty.store(true, Ordering::Relaxed);
    }

    /// 停止保护并落安全档；用于 SIGTERM / panic。
    pub fn shutdown_safe(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        let _ = apply_cap(&self.config, self.config.base_khz, false, &self.error);
        self.level.store(0, Ordering::Relaxed);
        self.cap_khz.store(self.config.base_khz, Ordering::Relaxed);
        self.boost.store(false, Ordering::Relaxed);
    }

    fn set_error(&self, message: impl Into<String>) {
        if let Ok(mut error) = self.error.lock() {
            *error = Some(message.into());
        }
    }
}

fn run(guard: &ThermalGuard) {
    let config = &guard.config;
    let mut level = 0i32;
    let mut last_enabled: Option<bool> = None;
    let mut last_manual: Option<(i32, bool)> = None;
    let mut last_change = Instant::now();
    let mut cool_since: Option<Instant> = None;
    let mut verify_countdown = 0u32;

    loop {
        verify_countdown += 1;
        if verify_countdown >= 5 {
            verify_countdown = 0;
            if guard.enabled.load(Ordering::Relaxed) {
                verify_caps(guard);
            }
        }
        if !guard.enabled.load(Ordering::Relaxed) {
            if last_enabled != Some(false) {
                let _ = apply_cap(config, config.base_khz, false, &guard.error);
                guard.level.store(0, Ordering::Relaxed);
                guard.cap_khz.store(config.base_khz, Ordering::Relaxed);
                guard.boost.store(false, Ordering::Relaxed);
                last_enabled = Some(false);
                last_manual = None;
            }
            std::thread::sleep(Duration::from_millis(config.sample_ms));
            continue;
        }

        if guard.manual.load(Ordering::SeqCst) {
            let state = guard
                .manual_state
                .lock()
                .map(|state| *state)
                .unwrap_or(ManualState {
                    level: -1,
                    cap: config.base_khz,
                    boost: false,
                });
            let dirty = guard.manual_dirty.swap(false, Ordering::SeqCst);
            if dirty || last_manual != Some((state.cap, state.boost)) {
                if state.boost && !guard.boost.load(Ordering::Relaxed) {
                    let _ = apply_cap(config, config.base_khz, true, &guard.error);
                }
                let _ = apply_cap(config, state.cap, state.boost, &guard.error);
                guard.cap_khz.store(state.cap, Ordering::Relaxed);
                guard.boost.store(state.boost, Ordering::Relaxed);
                guard.level.store(state.level, Ordering::Relaxed);
                last_manual = Some((state.cap, state.boost));
            }
            if let Some(temp) = read_temps().map(|temps| temps.0.max(temps.1).max(temps.2)) {
                guard.temp.store(temp, Ordering::Relaxed);
            }
            last_enabled = Some(true);
            std::thread::sleep(Duration::from_millis(config.sample_ms));
            continue;
        }
        last_manual = None;

        if last_enabled != Some(true) || guard.restart.swap(false, Ordering::Relaxed) {
            level = 0;
            last_change = Instant::now();
            cool_since = None;
            apply_level(guard, level);
            last_enabled = Some(true);
        }

        if let Some(temp) = read_temps().map(|temps| temps.0.max(temps.1).max(temps.2)) {
            guard.temp.store(temp, Ordering::Relaxed);
            let now = Instant::now();
            let next = next_level(
                level,
                temp,
                now,
                last_change,
                &mut cool_since,
                config,
            );
            if next != level {
                level = next;
                last_change = now;
                cool_since = None;
                apply_level(guard, level);
            }
        }

        std::thread::sleep(Duration::from_millis(config.sample_ms));
    }
}

/// 档位对应的配置频率与 boost 策略。
fn level_target(config: &ThermalConfig, level: i32, no_boost: bool) -> (i32, bool) {
    if no_boost {
        // 上次热断电后不允许 boost；高于基础频率的档位改用基础频率。
        return match level {
            2 => (config.cap2, false),
            3 => (config.cap3, false),
            _ => (config.base_khz, false),
        };
    }
    match level {
        1 => (config.cap1, true),
        2 => (config.cap2, false),
        3 => (config.cap3, false),
        _ => (config.cap0, true),
    }
}

/// 纯状态机：给定当前档位、温度和时间，返回下一档。
fn next_level(
    level: i32,
    temp: i32,
    now: Instant,
    last_change: Instant,
    cool_since: &mut Option<Instant>,
    config: &ThermalConfig,
) -> i32 {
    let target = if temp >= config.up3 {
        3
    } else if temp >= config.up2 {
        2
    } else if temp >= config.up1 {
        1
    } else {
        0
    };

    if target > level {
        return target;
    }
    if target == level {
        return level;
    }

    if level == 3 && !latch_ok(temp, now, cool_since, config) {
        return 3;
    }

    let down = match level {
        2 => config.down2,
        3 => config.down3,
        _ => config.down1,
    };
    if temp <= down
        && now.duration_since(last_change) >= Duration::from_millis(config.dwell_ms)
    {
        level - 1
    } else {
        level
    }
}

fn latch_ok(
    temp: i32,
    now: Instant,
    cool_since: &mut Option<Instant>,
    config: &ThermalConfig,
) -> bool {
    if temp <= config.latch_temp {
        if cool_since.is_none() {
            *cool_since = Some(now);
        }
    } else {
        *cool_since = None;
    }
    cool_since.is_some_and(|start| {
        now.duration_since(start) >= Duration::from_secs(config.latch_secs)
    })
}

fn apply_level(guard: &ThermalGuard, level: i32) {
    let (cap, boost) = level_target(&guard.config, level, guard.no_boost);
    // 从 boost 关闭切到开启时，先以基础频率打开 boost，再提高上限：
    // 否则内核会在 boost 尚未生效时把 scaling_max_freq 钳回基础频率。
    if boost && !guard.boost.load(Ordering::Relaxed) {
        let _ = apply_cap(
            &guard.config,
            guard.config.base_khz,
            true,
            &guard.error,
        );
    }
    if apply_cap(&guard.config, cap, boost, &guard.error).is_err() {
        // 错误已写入 guard.error；继续保留意图值供 UI 展示。
    }
    guard.level.store(level, Ordering::Relaxed);
    guard.cap_khz.store(cap, Ordering::Relaxed);
    guard.boost.store(boost, Ordering::Relaxed);
}

/// 写 cpufreq/EPP/boost 并读回校验。
fn apply_cap(
    _config: &ThermalConfig,
    cap_khz: i32,
    boost: bool,
    error: &Mutex<Option<String>>,
) -> Result<(), String> {
    let mut failures = Vec::new();
    let Ok(dir) = fs::read_dir(CPUFREQ) else {
        let message = format!("cannot read {CPUFREQ}");
        set_error(error, message.clone());
        return Err(message);
    };
    let mut policies: Vec<std::path::PathBuf> = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("policy"))
        {
            continue;
        }
        policies.push(path);
    }
    if policies.is_empty() {
        failures.push(format!("no cpufreq policies under {CPUFREQ}"));
    }
    let boost_path = format!("{CPUFREQ}/boost");
    let boost_value = if boost { "1" } else { "0" };
    // 开启 boost 必须先于提高频率：关闭 boost 时内核会把 scaling_max_freq
    // 钳到基础频率，先写高频再开 boost 会留下错误的低频上限。
    if boost {
        for _ in 0..2 {
            let _ = fs::write(&boost_path, boost_value);
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    // 写两遍再读回：systemd restart 时旧 helper 可能正在写安全档，
    // 单次立即读回会看到旧值，造成假错误。
    for _ in 0..2 {
        for path in &policies {
            // cap<=0 表示“不限制”：每个 policy 使用自己的硬件上限。
            let value = if cap_khz <= 0 {
                fs::read_to_string(path.join("cpuinfo_max_freq"))
                    .map(|value| value.trim().to_string())
                    .unwrap_or_else(|_| "0".to_string())
            } else {
                cap_khz.to_string()
            };
            if let Err(err) = fs::write(path.join("scaling_max_freq"), &value) {
                failures.push(format!("{}: {err}", path.join("scaling_max_freq").display()));
            }
            let _ = fs::write(path.join("energy_performance_preference"), "power");
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    if !boost {
        // 先降低频率，再关 boost，避免关闭瞬间被钳到更低值。
        for _ in 0..2 {
            let _ = fs::write(&boost_path, boost_value);
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    if failures.is_empty() {
        if let Ok(mut error) = error.lock() {
            *error = None;
        }
        Ok(())
    } else {
        let message = failures.join("; ");
        set_error(error, message.clone());
        Err(message)
    }
}

/// 周期性读回校验：只有期望值与实际 sysfs 不一致时才记录错误。
fn verify_caps(guard: &ThermalGuard) {
    let desired_cap = guard.cap_khz.load(Ordering::Relaxed);
    let desired_boost = guard.boost.load(Ordering::Relaxed);
    let mut failures = Vec::new();
    if let Ok(dir) = fs::read_dir(CPUFREQ) {
        for entry in dir.flatten() {
            let path = entry.path();
            if !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("policy"))
            {
                continue;
            }
            let max = path.join("scaling_max_freq");
            let expected = if desired_cap <= 0 {
                fs::read_to_string(path.join("cpuinfo_max_freq"))
                    .map(|value| value.trim().to_string())
                    .unwrap_or_else(|_| "0".to_string())
            } else {
                desired_cap.to_string()
            };
            match fs::read_to_string(&max) {
                Ok(value) if value.trim() == expected => {}
                Ok(value) => failures.push(format!(
                    "{}: expected {expected}, read {}",
                    max.display(),
                    value.trim()
                )),
                Err(err) => failures.push(format!("{}: {err}", max.display())),
            }
        }
    }
    let boost_path = format!("{CPUFREQ}/boost");
    match fs::read_to_string(&boost_path) {
        Ok(value) if value.trim() == if desired_boost { "1" } else { "0" } => {}
        Ok(value) => failures.push(format!(
            "{boost_path}: expected {}, read {}",
            if desired_boost { 1 } else { 0 },
            value.trim()
        )),
        Err(err) => failures.push(format!("{boost_path}: {err}")),
    }
    if failures.is_empty() {
        if let Ok(mut error) = guard.error.lock() {
            *error = None;
        }
    } else {
        set_error(&guard.error, failures.join("; "));
    }
}

fn set_error(error: &Mutex<Option<String>>, message: String) {
    if let Ok(mut slot) = error.lock() {
        if slot.as_deref() != Some(message.as_str()) {
            eprintln!("thermal guard: {message}");
        }
        *slot = Some(message);
    }
}

/// 返回 (Tctl, Tccd1, Tccd2)；缺失的传感器记为 0。
fn read_temps() -> Option<(i32, i32, i32)> {
    let dir = fs::read_dir("/sys/class/hwmon").ok()?;
    for entry in dir.flatten() {
        let base = entry.path();
        let Ok(name) = fs::read_to_string(base.join("name")) else {
            continue;
        };
        if name.trim() != "k10temp" {
            continue;
        }
        let mut tctl = 0;
        let mut tccd1 = 0;
        let mut tccd2 = 0;
        for index in 1..=6 {
            let Ok(label) = fs::read_to_string(base.join(format!("temp{index}_label"))) else {
                continue;
            };
            let Ok(raw) = fs::read_to_string(base.join(format!("temp{index}_input"))) else {
                continue;
            };
            let Ok(value) = raw.trim().parse::<i32>() else {
                continue;
            };
            let value = value / 1000;
            match label.trim() {
                "Tctl" => tctl = value,
                "Tccd1" => tccd1 = value,
                "Tccd2" => tccd2 = value,
                _ => {}
            }
        }
        return Some((tctl, tccd1, tccd2));
    }
    None
}

fn load_enabled() -> bool {
    fs::read_to_string(ENABLED_PATH)
        .or_else(|_| fs::read_to_string(LEGACY_ENABLED_PATH))
        .map(|value| value.trim() != "0")
        .unwrap_or(true)
}

/// 原子写入 `/var/lib/ecpilot` 下的状态文件。
fn write_state(path: &str, value: &str) -> Result<(), String> {
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, value).map_err(|error| error.to_string())?;
    fs::rename(&temp, path).map_err(|error| error.to_string())?;
    Ok(())
}

fn load_bool(path: &str, default: bool) -> bool {
    fs::read_to_string(path)
        .map(|value| value.trim() != "0")
        .unwrap_or(default)
}

fn load_i32(path: &str, default: i32) -> i32 {
    fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

fn max_cap_khz() -> i32 {
    // amd-pstate 的 cpuinfo_max_freq 会随 boost 开关变化；优先读取静态
    // 硬件上限，避免 boost 关闭时把用户请求的高频误钳到基础频率。
    fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/amd_pstate_max_freq")
        .or_else(|_| fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq"))
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(5_200_000)
}

/// 上次启动若是热断电，强制关闭 boost，避免重启后再次满载跳闸。
fn last_boot_thermal_trip() -> bool {
    Command::new("dmesg")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .contains("internal CPU thermal limit was tripped")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ThermalConfig {
        ThermalConfig::default()
    }

    #[test]
    fn raises_immediately_across_thresholds() {
        let cfg = config();
        let now = Instant::now();
        let mut cool = None;
        assert_eq!(next_level(0, 91, now, now, &mut cool, &cfg), 1);
        assert_eq!(next_level(1, 98, now, now, &mut cool, &cfg), 2);
        assert_eq!(next_level(2, 102, now, now, &mut cool, &cfg), 3);
    }

    #[test]
    fn downshift_needs_dwell_and_down_threshold() {
        let cfg = config();
        let now = Instant::now();
        let mut cool = None;
        assert_eq!(next_level(1, 86, now, now, &mut cool, &cfg), 1);
        assert_eq!(
            next_level(1, 84, now, now - Duration::from_secs(9), &mut cool, &cfg),
            0
        );
        assert_eq!(next_level(1, 84, now, now, &mut cool, &cfg), 1);
    }

    #[test]
    fn level_three_latches_until_cool_for_thirty_seconds() {
        let cfg = config();
        let start = Instant::now();
        let mut cool = None;
        assert_eq!(next_level(3, 80, start, start, &mut cool, &cfg), 3);
        assert_eq!(
            next_level(3, 80, start + Duration::from_secs(10), start, &mut cool, &cfg),
            3
        );
        assert_eq!(
            next_level(
                3,
                80,
                start + Duration::from_secs(31),
                start,
                &mut cool,
                &cfg
            ),
            2
        );
    }

    #[test]
    fn manual_levels_map_to_configured_caps() {
        let cfg = config();
        assert_eq!(level_target(&cfg, 0, false), (cfg.cap0, true));
        assert_eq!(level_target(&cfg, 1, false), (cfg.cap1, true));
        assert_eq!(level_target(&cfg, 2, false), (cfg.cap2, false));
        assert_eq!(level_target(&cfg, 3, false), (cfg.cap3, false));
        assert_eq!(level_target(&cfg, 0, true), (cfg.base_khz, false));
        assert_eq!(level_target(&cfg, 1, true), (cfg.base_khz, false));
    }

    #[test]
    fn config_env_parses_values() {
        // SAFETY: tests run single-threaded for this variable set.
        unsafe {
            std::env::set_var("MCC_THERMAL_UP1", "80");
            std::env::set_var("MCC_THERMAL_SAMPLE_MS", "500");
        }
        let cfg = ThermalConfig::from_env();
        assert_eq!(cfg.up1, 80);
        assert_eq!(cfg.sample_ms, 500);
        unsafe {
            std::env::remove_var("MCC_THERMAL_UP1");
            std::env::remove_var("MCC_THERMAL_SAMPLE_MS");
        }
    }
}
