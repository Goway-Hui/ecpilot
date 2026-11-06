//! MSI EC 与硬件访问层。
//!
//! 普通读取走 sysfs；需要 root 的写入通过系统 D-Bus 服务
//! `ecpilot.helper` 完成，GUI 本身不以 root 运行。

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const MSI_EC: &str = "/sys/devices/platform/msi-ec";

use crate::thermal_config::ThermalConfig;

#[derive(Clone, Default)]
pub struct Status {
    pub fw: String,
    pub fan_mode: String,
    pub shift_mode: String,
    pub cooler_boost: bool,
    pub cpu_temp: i32,
    pub gpu_temp: i32,
    pub cpu_fan: i32,
    pub gpu_fan: i32,
    pub battery: i32,
    pub charging: String,
    pub threshold: i32,
    pub gpu_name: String,
    pub gpu_power: f32,
    pub gpu_util: i32,
    pub gpu_temp_nv: i32,
    pub gpu_mem_used: i32,
    pub gpu_mem_total: i32,
    pub webcam: bool,
    pub usb_power_share: bool,
    pub fn_swapped: bool,
    /// k10temp Tctl（°C）。
    pub tctl: i32,
    /// k10temp Tccd1（°C）。
    pub tccd1: i32,
    /// k10temp Tccd2（°C，双 CCD 机型）。
    pub tccd2: i32,
    /// 内置温控是否启用。
    pub thermal_active: bool,
    /// 内置温控当前档位（0-3；-1 表示手动自定义频率）。
    pub thermal_level: i32,
    /// 当前档位的频率上限（MHz，0 表示不限制）。
    pub thermal_cap_mhz: i32,
    /// 当前是否允许 boost。
    pub thermal_boost: bool,
    /// 最近一次写 sysfs 的错误（空字符串表示正常）。
    pub thermal_error: String,
    /// 内置温控实际参数。
    pub thermal_config: ThermalConfig,
    /// AMD 核显温度（0 表示不可用）。
    pub igpu_temp: i32,
    /// AMD 核显功耗（W）。
    pub igpu_power: f32,
}

fn read_str(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_i32(path: &str) -> i32 {
    read_str(path).and_then(|s| s.parse().ok()).unwrap_or(0)
}

/// 一次枚举 `/sys/class/hwmon` 的 `(name, path)`，避免为每个传感器重复扫描。
fn hwmon_devices() -> Vec<(String, PathBuf)> {
    let mut devices = Vec::new();
    if let Ok(dir) = fs::read_dir("/sys/class/hwmon") {
        for entry in dir.flatten() {
            let base = entry.path();
            if let Some(name) = read_str(&base.join("name").to_string_lossy()) {
                devices.push((name, base));
            }
        }
    }
    devices
}

/// 读取 EC、电池、风扇与 GPU 状态。
///
/// GPU 快照有 2 秒缓存，避免每秒启动 `nvidia-smi`。
pub fn read_status() -> Status {
    let mut status = Status {
        fw: read_str(&format!("{MSI_EC}/fw_version")).unwrap_or_default(),
        fan_mode: read_str(&format!("{MSI_EC}/fan_mode")).unwrap_or_default(),
        shift_mode: read_str(&format!("{MSI_EC}/shift_mode")).unwrap_or_default(),
        cooler_boost: read_str(&format!("{MSI_EC}/cooler_boost")).as_deref() == Some("on"),
        cpu_temp: read_i32(&format!("{MSI_EC}/cpu/realtime_temperature")),
        gpu_temp: read_i32(&format!("{MSI_EC}/gpu/realtime_temperature")),
        ..Default::default()
    };

    let hwmon = hwmon_devices();

    if let Some((_, base)) = hwmon.iter().find(|(name, _)| name == "msi_wmi_platform") {
        status.cpu_fan = read_i32(&base.join("fan1_input").to_string_lossy());
        status.gpu_fan = read_i32(&base.join("fan2_input").to_string_lossy());
    }

    status.battery = read_i32("/sys/class/power_supply/BAT1/capacity");
    status.charging = read_str("/sys/class/power_supply/BAT1/status").unwrap_or_default();
    status.threshold = read_i32("/sys/class/power_supply/BAT1/charge_control_end_threshold");
    status.webcam = read_str(&format!("{MSI_EC}/webcam")).as_deref() == Some("on");
    status.fn_swapped = read_str(&format!("{MSI_EC}/fn_key")).as_deref() == Some("left");

    // k10temp：按 label 读取 Tctl / Tccd1 / Tccd2，兼容双 CCD 机型。
    if let Some((_, base)) = hwmon.iter().find(|(name, _)| name == "k10temp") {
        for index in 1..=6 {
            let label = read_str(&base.join(format!("temp{index}_label")).to_string_lossy());
            let value = read_i32(&base.join(format!("temp{index}_input")).to_string_lossy()) / 1000;
            match label.as_deref() {
                Some("Tctl") => status.tctl = value,
                Some("Tccd1") => status.tccd1 = value,
                Some("Tccd2") => status.tccd2 = value,
                _ => {}
            }
        }
    }

    if let Ok(proxy) = dbus_proxy() {
        let reply: Result<(bool, i32, i32, i32, bool, String), _> =
            proxy.call("thermalStatus", &());
        if let Ok((active, level, cap_khz, _temp, boost, error)) = reply {
            status.thermal_active = active;
            status.thermal_level = level;
            status.thermal_cap_mhz = cap_khz / 1000;
            status.thermal_boost = boost;
            status.thermal_error = error;
        }
        let config: Result<String, _> = proxy.call("thermalConfig", &());
        if let Ok(json) = config {
            if let Ok(parsed) = serde_json::from_str(&json) {
                status.thermal_config = parsed;
            }
        }
    }
    if let Some((_, base)) = hwmon.iter().find(|(name, _)| name == "amdgpu") {
        status.igpu_temp = read_i32(&base.join("temp1_input").to_string_lossy()) / 1000;
        status.igpu_power =
            read_i32(&base.join("power1_input").to_string_lossy()) as f32 / 1_000_000.0;
    }
    status.usb_power_share = dbus_usb_power_share().unwrap_or(false);

    let gpu = read_gpu();
    status.gpu_name = gpu.name;
    status.gpu_power = gpu.power;
    status.gpu_temp_nv = gpu.temp;
    status.gpu_util = gpu.util;
    status.gpu_mem_used = gpu.mem_used;
    status.gpu_mem_total = gpu.mem_total;

    status
}

#[derive(Clone, Default)]
struct GpuInfo {
    name: String,
    power: f32,
    temp: i32,
    util: i32,
    mem_used: i32,
    mem_total: i32,
}

static GPU_CACHE: Mutex<Option<(Instant, GpuInfo)>> = Mutex::new(None);

/// 读取 NVIDIA GPU 信息，结果缓存 2 秒。
fn read_gpu() -> GpuInfo {
    if let Ok(guard) = GPU_CACHE.lock() {
        if let Some((at, info)) = guard.as_ref() {
            if at.elapsed() < Duration::from_secs(2) {
                return info.clone();
            }
        }
    }
    let mut info = GpuInfo::default();
    if let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,power.draw,temperature.gpu,utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = text.lines().next().unwrap_or("").split(',').collect();
            if parts.len() >= 6 {
                info.name = parts[0].trim().to_string();
                info.power = parts[1].trim().parse().unwrap_or(0.0);
                info.temp = parts[2].trim().parse().unwrap_or(0);
                info.util = parts[3].trim().parse().unwrap_or(0);
                info.mem_used = parts[4].trim().parse().unwrap_or(0);
                info.mem_total = parts[5].trim().parse().unwrap_or(0);
            }
        }
    }
    if let Ok(mut guard) = GPU_CACHE.lock() {
        *guard = Some((Instant::now(), info.clone()));
    }
    info
}

/// Read the MSI MUX target from the UEFI variable (no root needed).
pub fn mux_mode() -> String {
    let Ok(dir) = fs::read_dir("/sys/firmware/efi/efivars") else {
        return "Unknown".into();
    };
    for entry in dir.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("MsiDCVarData-") {
            continue;
        }
        if let Ok(data) = fs::read(entry.path()) {
            if data.len() > 5 {
                return if data[5] == 1 { "Discrete".into() } else { "Hybrid".into() };
            }
        }
    }
    "Unknown".into()
}

/// Run a privileged helper operation through pkexec.
pub fn run_helper(args: &[&str]) -> Result<String> {
    let exe = std::env::current_exe().context("resolving own executable")?;
    let output = Command::new("pkexec")
        .arg(exe)
        .arg("--helper")
        .args(args)
        .output()
        .context("starting pkexec")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        bail!(if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 通过 root helper 设置风扇模式（auto/silent/advanced）。
pub fn set_fan_mode(mode: &str) -> Result<()> {
    dbus_proxy()?
        .call_method("setFanMode", &(mode,))
        .context("calling setFanMode")?;
    Ok(())
}

/// 通过 root helper 设置性能模式（eco/comfort/sport/turbo）。
pub fn set_shift_mode(mode: &str) -> Result<()> {
    dbus_proxy()?
        .call_method("setShiftMode", &(mode,))
        .context("calling setShiftMode")?;
    Ok(())
}

/// 开启/关闭 Cooler Boost。
pub fn set_cooler_boost(on: bool) -> Result<()> {
    dbus_put(0x98, if on { 0x86 } else { 0x06 })
}

/// 设置电池充电上限。
pub fn set_battery_threshold(value: u8) -> Result<()> {
    dbus_proxy()?
        .call_method("setBatteryThreshold", &(value as i32,))
        .context("calling setBatteryThreshold")?;
    Ok(())
}

pub fn set_usb_power_share(on: bool) -> Result<()> {
    dbus_proxy()?
        .call_method("setUsbPowerShare", &(on,))
        .context("calling setUsbPowerShare")?;
    Ok(())
}

pub fn set_fn_win_swap(on: bool) -> Result<()> {
    dbus_proxy()?
        .call_method("setFnWinSwap", &(on,))
        .context("calling setFnWinSwap")?;
    Ok(())
}

/// 应用一个预设风扇曲线。
pub fn apply_preset(name: &str) -> Result<()> {
    apply_curve_via_dbus(&preset(name)?)
}

pub fn apply_custom(
    cpu_temp: &[u8; 6],
    cpu_speed: &[u8; 7],
    gpu_temp: &[u8; 6],
    gpu_speed: &[u8; 7],
) -> Result<()> {
    let curve = Curve {
        cpu_temp: *cpu_temp,
        cpu_speed: *cpu_speed,
        gpu_temp: *gpu_temp,
        gpu_speed: *gpu_speed,
    };
    apply_curve_via_dbus(&curve)
}

/// 启用/停用 thermal-guard systemd 服务。
pub fn set_thermal_guard(on: bool) -> Result<()> {
    dbus_proxy()?
        .call_method("setThermalGuard", &(on,))
        .context("calling setThermalGuard")?;
    Ok(())
}

/// 重启 thermal-guard，重新读取 unit 中的档位参数。
pub fn restart_thermal_guard() -> Result<()> {
    dbus_proxy()?
        .call_method("restartThermalGuard", &())
        .context("calling restartThermalGuard")?;
    Ok(())
}

/// 保存档位规则（JSON，档位触发温度与频率上限）；`"{}"` 恢复默认。
pub fn set_thermal_rules(json: String) -> Result<()> {
    dbus_proxy()?
        .call_method("setThermalRules", &(json,))
        .context("calling setThermalRules")?;
    Ok(())
}

pub fn set_webcam(on: bool) -> Result<()> {
    dbus_proxy()?
        .call_method("setWebcam", &(on,))
        .context("calling setWebcam")?;
    Ok(())
}

pub fn klc_color(r: u8, g: u8, b: u8, brightness: u8) -> Result<()> {
    run_helper(&[
        "klc",
        &r.to_string(),
        &g.to_string(),
        &b.to_string(),
        &brightness.to_string(),
    ])
    .map(|_| ())
}

pub fn klc_off() -> Result<()> {
    run_helper(&["klc-off"]).map(|_| ())
}

pub fn set_mux(discrete: bool) -> Result<()> {
    run_helper(&["mux", if discrete { "dgpu" } else { "igpu" }]).map(|_| ())
}

fn dbus_connection() -> Result<&'static zbus::blocking::Connection> {
    static CONNECTION: OnceLock<zbus::blocking::Connection> = OnceLock::new();
    if let Some(connection) = CONNECTION.get() {
        return Ok(connection);
    }
    let connection = zbus::blocking::Connection::system().context("connecting to system bus")?;
    let _ = CONNECTION.set(connection);
    Ok(CONNECTION.get().expect("connection was just set"))
}

/// 创建到 root helper 的阻塞式 D-Bus 代理。
fn dbus_proxy() -> Result<zbus::blocking::Proxy<'static>> {
    zbus::blocking::Proxy::new(
        dbus_connection()?,
        "ecpilot.helper",
        "/",
        "org.ecpilot.Control",
    )
    .context("creating helper proxy")
}

fn dbus_put(address: u8, value: u8) -> Result<()> {
    dbus_proxy()?
        .call_method("putValue", &(address as i32, value as i32))
        .context("calling putValue")?;
    Ok(())
}

fn dbus_put_many(pairs: &[(u8, u8)]) -> Result<()> {
    let values: Vec<(i32, i32)> = pairs
        .iter()
        .map(|&(address, value)| (address as i32, value as i32))
        .collect();
    dbus_proxy()?
        .call_method("putValues", &(values,))
        .context("calling putValues")?;
    Ok(())
}

fn dbus_data() -> Result<Vec<u8>> {
    let data: Vec<u8> = dbus_proxy()?
        .call("getData", &())
        .context("calling getData")?;
    if data.len() < 256 {
        bail!("helper returned {} EC bytes", data.len());
    }
    Ok(data)
}

fn dbus_usb_power_share() -> Result<bool> {
    let enabled: bool = dbus_proxy()?
        .call("getUsbPowerShare", &())
        .context("calling getUsbPowerShare")?;
    Ok(enabled)
}

fn write_curve_dbus(curve: &Curve) -> Result<()> {
    let mut pairs = Vec::with_capacity(26);
    for index in 0..6 {
        pairs.push((0x6a + index as u8, curve.cpu_temp[index]));
        pairs.push((0x82 + index as u8, curve.gpu_temp[index]));
    }
    for index in 0..7 {
        pairs.push((0x72 + index as u8, curve.cpu_speed[index]));
        pairs.push((0x8a + index as u8, curve.gpu_speed[index]));
    }
    dbus_put_many(&pairs)
}

fn restore_curve_dbus(snapshot: &[u8]) -> Result<()> {
    let mut pairs = Vec::with_capacity(26);
    for (start, length) in [(0x6au8, 6usize), (0x72u8, 7), (0x82u8, 6), (0x8au8, 7)] {
        for index in 0..length {
            pairs.push((start + index as u8, snapshot[start as usize + index]));
        }
    }
    dbus_put_many(&pairs)
}

fn apply_curve_via_dbus(curve: &Curve) -> Result<()> {
    validate(&curve.cpu_temp, &curve.cpu_speed)?;
    validate(&curve.gpu_temp, &curve.gpu_speed)?;
    let snapshot = dbus_data()?;
    write_curve_dbus(curve)?;
    let readback = dbus_data()?;
    let ok = readback[0x6a..0x70] == curve.cpu_temp[..]
        && readback[0x72..0x79] == curve.cpu_speed[..]
        && readback[0x82..0x88] == curve.gpu_temp[..]
        && readback[0x8a..0x91] == curve.gpu_speed[..];
    if !ok {
        let _ = restore_curve_dbus(&snapshot);
        bail!("EC did not accept the curve; previous values restored");
    }
    set_fan_mode("advanced")?;
    Ok(())
}

struct Curve {
    cpu_temp: [u8; 6],
    cpu_speed: [u8; 7],
    gpu_temp: [u8; 6],
    gpu_speed: [u8; 7],
}

fn preset(name: &str) -> Result<Curve> {
    let curve = match name {
        "default" => Curve {
            cpu_temp: [50, 57, 65, 75, 85, 95],
            cpu_speed: [0, 40, 60, 91, 125, 140, 150],
            gpu_temp: [55, 60, 65, 70, 75, 80],
            gpu_speed: [0, 48, 55, 62, 68, 77, 85],
        },
        "quiet" => Curve {
            cpu_temp: [55, 62, 70, 78, 86, 95],
            cpu_speed: [0, 30, 45, 60, 75, 90, 110],
            gpu_temp: [60, 66, 72, 78, 84, 90],
            gpu_speed: [0, 30, 45, 60, 75, 90, 110],
        },
        "gaming" => Curve {
            cpu_temp: [45, 52, 60, 68, 75, 85],
            cpu_speed: [30, 45, 60, 75, 95, 120, 150],
            gpu_temp: [45, 52, 60, 68, 75, 85],
            gpu_speed: [30, 45, 60, 75, 95, 120, 150],
        },
        "max" => Curve {
            cpu_temp: [40, 50, 60, 70, 80, 90],
            cpu_speed: [150; 7],
            gpu_temp: [40, 50, 60, 70, 80, 90],
            gpu_speed: [150; 7],
        },
        other => bail!("unknown preset {other:?}"),
    };
    validate(&curve.cpu_temp, &curve.cpu_speed)?;
    validate(&curve.gpu_temp, &curve.gpu_speed)?;
    Ok(curve)
}

fn validate(temp: &[u8; 6], speed: &[u8; 7]) -> Result<()> {
    for (i, &t) in temp.iter().enumerate() {
        if !(40..=95).contains(&t) {
            bail!("temperature threshold {} is {t} °C", i + 1);
        }
        if i > 0 && t <= temp[i - 1] {
            bail!("temperature thresholds must be strictly increasing");
        }
    }
    for (i, &s) in speed.iter().enumerate() {
        if s > 150 {
            bail!("fan speed {} is {s} % (max 150)", i + 1);
        }
        if i > 0 && s < speed[i - 1] {
            bail!("fan speeds must be non-decreasing");
        }
    }
    if speed[6] < 70 {
        bail!("last fan speed must be at least 70 %");
    }
    Ok(())
}

/// Root-side entry point for `pkexec ecpilot --helper ...`.
///
/// 只保留必须由 GUI 进程外部提权的操作；EC/sysfs 写入统一走 D-Bus helper。
pub fn helper_main(args: &[String]) -> Result<()> {
    let Some(command) = args.first().map(String::as_str) else {
        bail!("missing helper command");
    };
    match command {
        "klc" => {
            let r = args.get(1).context("r")?.parse()?;
            let g = args.get(2).context("g")?.parse()?;
            let b = args.get(3).context("b")?.parse()?;
            let brightness = args.get(4).map(|s| s.parse()).transpose()?.unwrap_or(255);
            crate::klc::apply_rgb(r, g, b, brightness)
        }
        "klc-off" => crate::klc::off(),
        "mux" => {
            let mode = args.get(1).context("mode")?;
            let output = Command::new("/usr/local/bin/msi-gpu-switcher")
                .arg(mode)
                .output()
                .context("running msi-gpu-switcher")?;
            if !output.status.success() {
                bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
            }
            Ok(())
        }
        other => bail!("unknown helper command {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_validate() {
        for name in ["default", "quiet", "gaming", "max"] {
            assert!(preset(name).is_ok(), "preset {name} should validate");
        }
    }

    #[test]
    fn rejects_non_increasing_temperatures() {
        let temp = [50, 50, 65, 75, 85, 95];
        let speed = [0, 40, 60, 91, 125, 140, 150];
        assert!(validate(&temp, &speed).is_err());
    }

    #[test]
    fn rejects_out_of_range_speed() {
        let temp = [50, 57, 65, 75, 85, 95];
        let speed = [0, 40, 60, 91, 125, 140, 151];
        assert!(validate(&temp, &speed).is_err());
    }

}
