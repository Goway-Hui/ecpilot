//! Frame builders for the MSI SteelSeries "KLC" per-key RGB protocol.
//!
//! ECPilot only exposes solid colors today, so this module carries exactly
//! the frames that path needs: one `0x0E` group frame per region group, then
//! a `0x0D` commit. The wider command table (vendor commands, free-form
//! `0x0C`, onboard effect programs) lives in the upstream `msi-klc` project;
//! it can be reintroduced together with the UI that uses it.
//!
//! Byte layouts follow MSI Center's frame formats as documented in
//! `KLC-PROTOCOL.md` and validated on hardware (MS-16V5 / GS66, `1038:113a`).

/// Length of a KLC HID *feature* report, including the leading report-id byte
/// (`buf[0] = 0`, since the device uses unnumbered reports). The wire payload
/// is 524 bytes; the ioctl buffer is 525 bytes total.
pub const FEATURE_REPORT_LEN: usize = 525;

/// Length of a KLC HID *output*/*input* report, including the leading
/// report-id byte.
pub const OUTPUT_REPORT_LEN: usize = 64;

/// Max per-key color entries in one "steady/group" (`0x0E`) feature report.
pub const MAX_GROUP_ENTRIES: usize = (FEATURE_REPORT_LEN - 5) / 12;

/// Vendor command bytes used by the solid-color path.
pub mod cmd {
    /// One `0x0E` feature report per region group.
    pub const SET_COLOR_GROUP: u8 = 0x0E;
    /// Push the RAM color buffer to the live LEDs.
    pub const SHOW: u8 = 0x0D;
}

/// `settings_mask` value for a fixed `init` color.
const SETTINGS_MASK_STEADY: u8 = 1;

/// The legacy react-fade-time value (`0x012C` = 300 ms) that the original
/// encoding always sent even though `settings_mask=STEADY` never uses the
/// reactive fields. Preserved byte-for-byte for compatibility.
const STEADY_LEGACY_REACT_TIME_MS: u16 = 0x012C;

/// One per-key element of a `0x0E` "group" feature report: 12 bytes.
///
/// ```text
/// [ init.R, init.G, init.B,
///   react.R, react.G, react.B,
///   react.time_lo, react.time_hi,
///   effect_index, settings_mask, lockmask, hid ]
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyElement {
    pub init: [u8; 3],
    pub react: [u8; 3],
    pub react_time_ms: u16,
    pub effect_index: u8,
    pub settings_mask: u8,
    pub lockmask: u8,
    pub hid: u8,
}

impl KeyElement {
    /// Serialize to the 12-byte on-wire element.
    pub fn to_bytes(self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0..3].copy_from_slice(&self.init);
        buf[3..6].copy_from_slice(&self.react);
        buf[6..8].copy_from_slice(&self.react_time_ms.to_le_bytes());
        buf[8] = self.effect_index;
        buf[9] = self.settings_mask;
        buf[10] = self.lockmask;
        buf[11] = self.hid;
        buf
    }

    /// A fixed/static color element: `init = (r, g, b)`, no reactive
    /// behavior, `settings_mask = STEADY`.
    pub fn steady(hid: u8, r: u8, g: u8, b: u8) -> Self {
        Self {
            init: [r, g, b],
            react: [0, 0, 0],
            react_time_ms: STEADY_LEGACY_REACT_TIME_MS,
            effect_index: 0,
            settings_mask: SETTINGS_MASK_STEADY,
            lockmask: 0,
            hid,
        }
    }
}

/// Build one "steady/group" (`0x0E`) feature report from [`KeyElement`]s.
///
/// Panics if `elements.len() > MAX_GROUP_ENTRIES`.
pub fn build_group_frame_elements(elements: &[KeyElement]) -> [u8; FEATURE_REPORT_LEN] {
    assert!(
        elements.len() <= MAX_GROUP_ENTRIES,
        "group has {} keys, max {} fit in one 0x0E report",
        elements.len(),
        MAX_GROUP_ENTRIES
    );
    let mut buf = [0u8; FEATURE_REPORT_LEN];
    buf[1] = cmd::SET_COLOR_GROUP;
    buf[3] = elements.len() as u8;
    for (m, e) in elements.iter().enumerate() {
        let base = 5 + 12 * m;
        buf[base..base + 12].copy_from_slice(&e.to_bytes());
    }
    buf
}

/// Build one `0x0E` feature report for a region group, every key set to the
/// same `(r, g, b)`.
pub fn build_group_frame(hids: &[u8], r: u8, g: u8, b: u8) -> [u8; FEATURE_REPORT_LEN] {
    let elements: Vec<KeyElement> = hids
        .iter()
        .map(|&hid| KeyElement::steady(hid, r, g, b))
        .collect();
    build_group_frame_elements(&elements)
}

/// Build the 64-byte "show"/commit output report:
/// `[0x00, 0x0D, 0x00, 0x02, 0...]`.
pub fn build_commit() -> [u8; OUTPUT_REPORT_LEN] {
    let mut buf = [0u8; OUTPUT_REPORT_LEN];
    buf[1] = cmd::SHOW;
    buf[3] = 0x02;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_frame_is_525_bytes() {
        let f = build_group_frame(&[41, 43, 53], 0x11, 0x22, 0x33);
        assert_eq!(f.len(), FEATURE_REPORT_LEN);
    }

    #[test]
    fn group_frame_header_and_entries() {
        let hids = [41u8, 43, 53];
        let f = build_group_frame(&hids, 0x10, 0x20, 0x30);
        assert_eq!(f[0], 0x00); // report id
        assert_eq!(f[1], cmd::SET_COLOR_GROUP);
        assert_eq!(f[3], hids.len() as u8);
        for (m, &hid) in hids.iter().enumerate() {
            let base = 5 + 12 * m;
            assert_eq!(f[base], 0x10);
            assert_eq!(f[base + 1], 0x20);
            assert_eq!(f[base + 2], 0x30);
            assert_eq!(f[base + 3], 0);
            assert_eq!(f[base + 4], 0);
            assert_eq!(f[base + 5], 0);
            assert_eq!(f[base + 6], 0x2C);
            assert_eq!(f[base + 7], 1);
            assert_eq!(f[base + 8], 0);
            assert_eq!(f[base + 9], 1);
            assert_eq!(f[base + 10], 0);
            assert_eq!(f[base + 11], hid);
        }
    }

    #[test]
    fn group_frame_all_black_zeroes_rgb_but_keeps_footer() {
        let f = build_group_frame(&[7], 0, 0, 0);
        assert_eq!(&f[5..17], &[0, 0, 0, 0, 0, 0, 0x2C, 1, 0, 1, 0, 7]);
    }

    #[test]
    #[should_panic]
    fn group_frame_rejects_oversize_group() {
        let hids: Vec<u8> = (0..=MAX_GROUP_ENTRIES as u16).map(|x| x as u8).collect();
        let _ = build_group_frame(&hids, 1, 2, 3);
    }

    /// Byte-compat guard: the old hand-rolled per-entry layout (transcribed
    /// here verbatim) must still match `KeyElement::steady` +
    /// `build_group_frame`.
    #[test]
    fn all_key_off_byte_identical_to_legacy_encoding() {
        fn legacy_group_frame(hids: &[u8], r: u8, g: u8, b: u8) -> [u8; FEATURE_REPORT_LEN] {
            let mut buf = [0u8; FEATURE_REPORT_LEN];
            buf[1] = cmd::SET_COLOR_GROUP;
            buf[3] = hids.len() as u8;
            for (m, &hid) in hids.iter().enumerate() {
                let base = 5 + 12 * m;
                buf[base] = r;
                buf[base + 1] = g;
                buf[base + 2] = b;
                // buf[base+3..base+6] (react rgb) stay 0
                buf[base + 6] = 0x2C;
                buf[base + 7] = 1;
                // buf[base+8] (effect_index) stays 0
                buf[base + 9] = 1; // settings_mask = STEADY
                // buf[base+10] (lockmask) stays 0
                buf[base + 11] = hid;
            }
            buf
        }

        let hids = [41u8, 43, 53, 7];
        for &(r, g, b) in &[(0xFFu8, 0x00u8, 0x00u8), (0, 0, 0), (0x11, 0x22, 0x33)] {
            assert_eq!(
                build_group_frame(&hids, r, g, b),
                legacy_group_frame(&hids, r, g, b),
                "new KeyElement-based encoding must be byte-identical to the legacy one"
            );
        }
    }

    #[test]
    fn commit_frame_matches_reference() {
        let f = build_commit();
        assert_eq!(f.len(), OUTPUT_REPORT_LEN);
        assert_eq!(&f[0..4], &[0x00, 0x0D, 0x00, 0x02]);
        assert!(f[4..].iter().all(|&b| b == 0));
    }
}
