//! hidraw device discovery + I/O for the MSI KLC keyboard controller.
//!
//! Detection scans `/sys/class/hidraw/*/device/uevent` for a `HID_ID` line
//! matching `0003:0000<VID>:0000<PID>` (bus=USB), which is how the kernel
//! exposes VID/PID for hidraw nodes without needing libudev.

use anyhow::{Context, Result, bail};
use nix::ioctl_readwrite;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

use crate::klc::protocol::FEATURE_REPORT_LEN;

pub const VID_MSI: u16 = 0x1038;

// HIDIOCSFEATURE(len) = _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, len). The ioctl
// number bakes in the buffer size, so we generate one wrapper sized exactly
// to our fixed 525-byte feature report.
ioctl_readwrite!(hidiocsfeature_525, b'H', 0x06, [u8; FEATURE_REPORT_LEN]);

/// Find a hidraw node under `/sys/class/hidraw` whose `HID_ID` matches
/// `vid` and (if given) one of `pids`. Returns the first match as
/// `(/dev/hidrawN, matched_pid)` — the matched PID lets the caller resolve
/// which model (of possibly many sharing `vid`) it just found, via
/// `models::ModelTable::find`.
pub fn find_device(vid: u16, pids: Option<&[u16]>) -> Result<(PathBuf, u16)> {
    let dir = fs::read_dir("/sys/class/hidraw").context("reading /sys/class/hidraw (is hidraw loaded?)")?;
    for entry in dir {
        let entry = entry?;
        let uevent_path = entry.path().join("device/uevent");
        let Ok(content) = fs::read_to_string(&uevent_path) else {
            continue;
        };
        let Some((found_vid, found_pid)) = parse_hid_id(&content) else {
            continue;
        };
        if found_vid == vid && pids.is_none_or(|list| list.contains(&found_pid)) {
            return Ok((PathBuf::from("/dev").join(entry.file_name()), found_pid));
        }
    }
    let pid_desc = match pids {
        None => "any".to_string(),
        Some([p]) => format!("0x{p:04x}"),
        Some(list) => format!("one of {} known KLC PIDs", list.len()),
    };
    bail!(
        "no /dev/hidraw* device found for vid=0x{vid:04x} pid={pid_desc}. Pass --path /dev/hidrawN \
         explicitly, or check `grep HID_ID /sys/class/hidraw/*/device/uevent`."
    )
}

/// Parse `HID_ID=0003:00001038:0000113A` into `(vid, pid)`. The bus type
/// (0003 = USB) is ignored; only the low 16 bits of the vendor/product
/// fields carry the actual VID/PID.
fn parse_hid_id(uevent: &str) -> Option<(u16, u16)> {
    let line = uevent.lines().find(|l| l.starts_with("HID_ID="))?;
    let rest = line.strip_prefix("HID_ID=")?;
    let mut parts = rest.split(':');
    let _bus = parts.next()?;
    let vid_field = parts.next()?;
    let pid_field = parts.next()?;
    let vid = u32::from_str_radix(vid_field, 16).ok()? as u16;
    let pid = u32::from_str_radix(pid_field, 16).ok()? as u16;
    Some((vid, pid))
}

/// An open hidraw device.
pub struct Device {
    file: File,
}

impl Device {
    pub fn open(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .with_context(|| format!("opening {} (needs root)", path.display()))?;
        Ok(Self { file })
    }

    /// Send a 525-byte feature report via `HIDIOCSFEATURE`.
    pub fn send_feature(&self, buf: &[u8; FEATURE_REPORT_LEN]) -> Result<()> {
        let mut buf = *buf;
        unsafe { hidiocsfeature_525(self.file.as_raw_fd(), &mut buf as *mut _) }
            .context("HIDIOCSFEATURE ioctl failed")?;
        Ok(())
    }

    /// Write an output report (commit/"show" frame) via a plain `write()`.
    pub fn write_output(&self, buf: &[u8]) -> Result<()> {
        (&self.file).write_all(buf).context("writing output report")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hid_id_uevent_line() {
        let uevent = "DRIVER=hid-generic\nHID_ID=0003:00001038:0000113A\nHID_NAME=foo\n";
        assert_eq!(parse_hid_id(uevent), Some((0x1038, 0x113a)));
    }

    #[test]
    fn missing_hid_id_returns_none() {
        assert_eq!(parse_hid_id("DRIVER=hid-generic\n"), None);
    }
}
