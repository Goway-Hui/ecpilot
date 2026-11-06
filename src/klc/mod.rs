//! MSI SteelSeries KLC 单键 RGB 支持。
//!
//! 协议、hidraw 发现、机型/布局表均从 MIT/Apache-2.0 的 `msi-klc`
//! 移植而来，本文件只暴露控制中心需要的操作。

pub mod device;
pub mod layout;
pub mod models;
pub mod protocol;

use anyhow::Result;
use std::time::Duration;

const SEND_DELAY: Duration = Duration::from_millis(10);

/// Apply one solid color to every key. `brightness` is 0..=255 and is folded
/// into the per-model color scale (software-owned brightness).
pub fn apply_rgb(r: u8, g: u8, b: u8, brightness: u8) -> Result<()> {
    let table = models::ModelTable::load()?;
    let pids = table.msi_pids();
    let (path, pid) = device::find_device(device::VID_MSI, Some(&pids))?;
    let scale = table
        .find(device::VID_MSI, pid)
        .map(|model| model.color_scale)
        .unwrap_or([1.0, 1.0, 1.0]);
    let factor = brightness as f32 / 255.0;
    let scaled = models::scale_rgb(
        [scale[0] * factor, scale[1] * factor, scale[2] * factor],
        (r, g, b),
    );

    let device = device::Device::open(&path)?;
    let layout = layout::Layout::load()?;
    for group in layout.groups()?.iter() {
        if group.is_empty() {
            continue;
        }
        let frame = protocol::build_group_frame(group, scaled.0, scaled.1, scaled.2);
        device.send_feature(&frame)?;
        std::thread::sleep(SEND_DELAY);
    }
    device.write_output(&protocol::build_commit())?;
    Ok(())
}

pub fn off() -> Result<()> {
    apply_rgb(0, 0, 0, 255)
}

/// Detection used by the UI without opening the device.
pub fn detect() -> Option<(String, String)> {
    let table = models::ModelTable::load().ok()?;
    let pids = table.msi_pids();
    let (path, pid) = device::find_device(device::VID_MSI, Some(&pids)).ok()?;
    let name = table
        .find(device::VID_MSI, pid)
        .map(|model| model.name.clone())
        .unwrap_or_else(|| "unknown KLC".to_string());
    Some((path.display().to_string(), name))
}
