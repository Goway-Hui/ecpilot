//! Region-group layout data, embedded at compile time from
//! `data/msi-layouts.json` (extracted from MSI Center's
//! `MysticLight_AllDevice.dll` by `../extract-msi-layouts.py`).
//!
//! `region_groups.GroupN_Offset` is the 6-way partition of HID usage codes
//! that the firmware's `Set_Keyboard_Color` groups keys into for the `0x0E`
//! bulk-color feature report. The JSON also carries per-model keymaps; the
//! solid-color path does not need them, so only the region groups are parsed.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// Raw embedded layout data (6 region groups).
const MSI_LAYOUTS_JSON: &str = include_str!("data/msi-layouts.json");

#[derive(Debug, Deserialize)]
struct LayoutsFile {
    region_groups: HashMap<String, Vec<u32>>,
}

/// Parsed layout data plus convenience lookups.
pub struct Layout {
    file: LayoutsFile,
}

impl Layout {
    /// Parse the embedded `msi-layouts.json`.
    pub fn load() -> Result<Self> {
        let file: LayoutsFile =
            serde_json::from_str(MSI_LAYOUTS_JSON).context("parsing embedded msi-layouts.json")?;
        Ok(Self { file })
    }

    /// The six region groups, in `Group1_Offset..Group6_Offset` order, each
    /// a list of HID usage codes. One `0x0E` report is built per group —
    /// sending all six is required to reach every key (including keys like
    /// Power that a naive single-group approach misses).
    pub fn groups(&self) -> Result<[Vec<u8>; 6]> {
        let mut out: [Vec<u8>; 6] = Default::default();
        for (i, slot) in out.iter_mut().enumerate() {
            let key = format!("Group{}_Offset", i + 1);
            let group = self
                .file
                .region_groups
                .get(&key)
                .with_context(|| format!("missing {key} in msi-layouts.json"))?;
            *slot = group
                .iter()
                .map(|&hid| hid_to_u8(hid))
                .collect::<Result<Vec<u8>>>()?;
        }
        Ok(out)
    }
}

fn hid_to_u8(hid: u32) -> Result<u8> {
    u8::try_from(hid).map_err(|_| anyhow::anyhow!("HID usage code {hid} out of u8 range"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_embedded_layout() {
        assert!(Layout::load().is_ok());
    }

    #[test]
    fn groups_cover_the_expected_key_counts() {
        let layout = Layout::load().unwrap();
        let groups = layout.groups().unwrap();
        let total: usize = groups.iter().map(|group| group.len()).sum();
        // The six groups together hold 106 keys (GE73Keys superset).
        assert_eq!(total, 106);
    }
}
