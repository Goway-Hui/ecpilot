//! root 权限 D-Bus helper。
//!
//! GUI 以普通用户运行，所有 EC/sysfs 写入都通过本服务完成。

use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const EC_IO_PATH: &str = "/sys/kernel/debug/ec/ec0/io";
pub const MSI_EC_PATH: &str = "/sys/devices/platform/msi-ec";

/// EC registers the helper is allowed to write. Everything else is rejected.
/// 允许写入的 EC 寄存器白名单；其它地址一律拒绝。
pub fn ec_address_allowed(address: i32) -> bool {
    matches!(
        address,
        0x2c | 0x2e | 0x98 | 0xbf | 0xd3 | 0xe8 | 0xf2 | 0xf3 | 0xf4
    ) || (0x6a..=0x6f).contains(&address)
        || (0x72..=0x78).contains(&address)
        || (0x82..=0x87).contains(&address)
        || (0x8a..=0x90).contains(&address)
}

/// EC 0xBF 的 USB Power Share 位：两个位同时置位表示开启。
pub fn usb_power_share_enabled(current: u8) -> bool {
    current & 0x28 == 0x28
}

/// 读改写 EC 0xBF：保留同一字节内的其它位域（如 Fn/Win 交换位）。
pub fn usb_power_share_byte(current: u8, on: bool) -> u8 {
    let mut value = current & !0x28;
    if on {
        value |= 0x28;
    } else {
        value |= 0x08;
    }
    value
}

fn failed(error: impl std::fmt::Display) -> zbus::fdo::Error {
    zbus::fdo::Error::Failed(error.to_string())
}

fn ec_open() -> zbus::fdo::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(EC_IO_PATH)
        .map_err(failed)
}

/// 读取单个 EC 寄存器；用于读改写。
fn ec_read_byte(address: i32) -> Option<u8> {
    let mut file = std::fs::File::open(EC_IO_PATH).ok()?;
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Start(address as u64)).ok()?;
    let mut data = [0u8; 1];
    use std::io::Read;
    file.read_exact(&mut data).ok()?;
    Some(data[0])
}

fn validate_pair(address: i32, value: i32) -> zbus::fdo::Result<()> {
    if !ec_address_allowed(address) || !(0..=255).contains(&value) {
        return Err(failed(
            "EC address is not allowed or value out of range",
        ));
    }
    Ok(())
}

fn ec_write_byte(address: i32, value: i32) -> zbus::fdo::Result<()> {
    validate_pair(address, value)?;
    let mut file = ec_open()?;
    file.seek(SeekFrom::Start(address as u64)).map_err(failed)?;
    file.write_all(&[value as u8]).map_err(failed)?;
    file.flush().map_err(failed)
}

/// 一次打开 EC 设备并写入多个寄存器，减少 I/O 往返。
fn ec_write_pairs(pairs: &[(i32, i32)]) -> zbus::fdo::Result<()> {
    for &(address, value) in pairs {
        validate_pair(address, value)?;
    }
    let mut file = ec_open()?;
    for &(address, value) in pairs {
        file.seek(SeekFrom::Start(address as u64)).map_err(failed)?;
        file.write_all(&[value as u8]).map_err(failed)?;
    }
    file.flush().map_err(failed)
}

fn ec_read_all() -> zbus::fdo::Result<Vec<u8>> {
    let mut file = std::fs::File::open(EC_IO_PATH).map_err(failed)?;
    let mut data = vec![0u8; 256];
    file.read_exact(&mut data).map_err(failed)?;
    Ok(data)
}

/// 通过 msi-ec sysfs 交换 Fn/Win 键。
fn set_fn_win_swap(on: bool) -> zbus::fdo::Result<()> {
    let (fn_key, win_key) = if on {
        ("left", "right")
    } else {
        ("right", "left")
    };
    std::fs::write("/sys/devices/platform/msi-ec/win_key", win_key).map_err(failed)?;
    std::fs::write("/sys/devices/platform/msi-ec/fn_key", fn_key).map_err(failed)
}

#[path = "thermal.rs"]
mod thermal;

struct EcHelper {
    thermal: Arc<thermal::ThermalGuard>,
}

#[zbus::interface(name = "org.ecpilot.Control")]
impl EcHelper {
    #[zbus(name = "putValue")]
    fn put_value(&self, address: i32, value: i32) -> zbus::fdo::Result<()> {
        ec_write_byte(address, value)
    }

    #[zbus(name = "putValues")]
    fn put_values(&self, values: Vec<(i32, i32)>) -> zbus::fdo::Result<()> {
        ec_write_pairs(&values)
    }

    #[zbus(name = "getData")]
    fn get_data(&self) -> zbus::fdo::Result<Vec<u8>> {
        ec_read_all()
    }

    #[zbus(name = "isEcSysModuleLoaded")]
    fn is_ec_sys_module_loaded(&self) -> bool {
        std::path::Path::new(EC_IO_PATH).exists()
    }

    #[zbus(name = "loadEcSysModule")]
    fn load_ec_sys_module(&self) -> bool {
        let _ = std::process::Command::new("modprobe")
            .args(["ec_sys", "write_support=1"])
            .status();
        std::path::Path::new(EC_IO_PATH).exists()
    }

    #[zbus(name = "setFanMode")]
    fn set_fan_mode(&self, mode: String) -> zbus::fdo::Result<()> {
        if !matches!(mode.as_str(), "auto" | "silent" | "advanced") {
            return Err(failed("unknown fan mode"));
        }
        std::fs::write(format!("{MSI_EC_PATH}/fan_mode"), mode).map_err(failed)
    }

    #[zbus(name = "setShiftMode")]
    fn set_shift_mode(&self, mode: String) -> zbus::fdo::Result<()> {
        if !matches!(mode.as_str(), "eco" | "comfort" | "sport" | "turbo") {
            return Err(failed("unknown shift mode"));
        }
        std::fs::write(format!("{MSI_EC_PATH}/shift_mode"), mode).map_err(failed)
    }

    #[zbus(name = "setThermalGuard")]
    fn set_thermal_guard(&self, on: bool) -> zbus::fdo::Result<()> {
        self.thermal.set_enabled(on).map_err(failed)
    }

    #[zbus(name = "restartThermalGuard")]
    fn restart_thermal_guard(&self) -> zbus::fdo::Result<()> {
        self.thermal.restart();
        Ok(())
    }

    #[zbus(name = "setThermalRules")]
    fn set_thermal_rules(&self, json: String) -> zbus::fdo::Result<()> {
        self.thermal.set_rules(&json).map_err(failed)
    }

    #[zbus(name = "thermalStatus")]
    fn thermal_status(&self) -> (bool, i32, i32, i32, bool, String) {
        self.thermal.status()
    }

    #[zbus(name = "thermalConfig")]
    fn thermal_config(&self) -> String {
        self.thermal.config_json()
    }

    #[zbus(name = "setBatteryThreshold")]
    fn set_battery_threshold(&self, value: i32) -> zbus::fdo::Result<()> {
        if !(40..=100).contains(&value) {
            return Err(failed("battery threshold out of range"));
        }
        std::fs::write(
            "/sys/class/power_supply/BAT1/charge_control_end_threshold",
            value.to_string(),
        )
        .map_err(failed)
    }

    #[zbus(name = "setWebcam")]
    fn set_webcam(&self, on: bool) -> zbus::fdo::Result<()> {
        std::fs::write(
            "/sys/devices/platform/msi-ec/webcam",
            if on { "on" } else { "off" },
        )
        .map_err(failed)
    }

    #[zbus(name = "setUsbPowerShare")]
    fn set_usb_power_share(&self, on: bool) -> zbus::fdo::Result<()> {
        let current =
            ec_read_byte(0xbf).ok_or_else(|| failed("cannot read EC 0xbf for usb power share"))?;
        ec_write_byte(0xbf, usb_power_share_byte(current, on) as i32)
    }

    #[zbus(name = "getUsbPowerShare")]
    fn get_usb_power_share(&self) -> zbus::fdo::Result<bool> {
        let current =
            ec_read_byte(0xbf).ok_or_else(|| failed("cannot read EC 0xbf for usb power share"))?;
        Ok(usb_power_share_enabled(current))
    }

    /// 通过 msi-ec sysfs 交换 Fn/Win 键。
    #[zbus(name = "setFnWinSwap")]
    fn set_fn_win_swap(&self, on: bool) -> zbus::fdo::Result<()> {
        set_fn_win_swap(on)
    }
}

/// 启动 system D-Bus 服务并常驻。
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_shutdown_signal(_: i32) {
    SHUTDOWN.store(true, Ordering::Relaxed);
}

fn install_shutdown_signals() {
    use nix::sys::signal::{signal, SigHandler, Signal};
    // SAFETY: handler only sets an atomic flag.
    unsafe {
        let _ = signal(Signal::SIGTERM, SigHandler::Handler(handle_shutdown_signal));
        let _ = signal(Signal::SIGINT, SigHandler::Handler(handle_shutdown_signal));
    }
}

pub fn run() -> anyhow::Result<()> {
    install_shutdown_signals();
    let thermal = thermal::ThermalGuard::start();
    let _connection = zbus::blocking::connection::Builder::system()?
        .name("ecpilot.helper")?
        .serve_at("/", EcHelper { thermal: Arc::clone(&thermal) })?
        .build()?;
    loop {
        if SHUTDOWN.load(Ordering::Relaxed) {
            thermal.shutdown_safe();
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitelist_allows_known_registers() {
        for address in [0x2c, 0x2e, 0x98, 0xbf, 0xd3, 0xe8, 0xf2, 0xf3, 0xf4] {
            assert!(ec_address_allowed(address), "0x{address:02x} should be allowed");
        }
        for address in [0x6a, 0x6f, 0x72, 0x78, 0x82, 0x87, 0x8a, 0x90] {
            assert!(ec_address_allowed(address), "0x{address:02x} should be allowed");
        }
    }

    #[test]
    fn usb_power_share_preserves_other_bits() {
        assert_eq!(usb_power_share_byte(0x00, true), 0x28);
        assert_eq!(usb_power_share_byte(0x01, true), 0x29);
        assert_eq!(usb_power_share_byte(0x28, false), 0x08);
        assert_eq!(usb_power_share_byte(0x29, false), 0x09);
        assert!(usb_power_share_enabled(usb_power_share_byte(0x01, true)));
        assert!(!usb_power_share_enabled(usb_power_share_byte(0x29, false)));
    }

    #[test]
    fn whitelist_rejects_unknown_registers() {
        for address in [0x00, 0x69, 0x70, 0x79, 0x88, 0x91, 0xff] {
            assert!(!ec_address_allowed(address), "0x{address:02x} should be rejected");
        }
    }
}
