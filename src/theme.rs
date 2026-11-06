//! 主题与配色定义。
//!
//! 深色/浅色两套配色集中在这里，界面代码只通过 `Theme` 字段取色。
//! 设计约束：界面以中性色为主，强调色只用于选中态、主操作和焦点；
//! 状态色（green/yellow/red）只表达温度与健康度。

/// 全局字体；Noto Sans 在目标机器上覆盖拉丁与 CJK 回退。
pub(crate) const FONT_FAMILY: &str = "Noto Sans";

/// 卡片圆角。
pub(crate) const RADIUS_CARD: f32 = 12.0;
/// 控件圆角。
pub(crate) const RADIUS_CONTROL: f32 = 8.0;

#[derive(Clone, Copy, PartialEq)]
/// 主题模式：深色 / 浅色。
pub(crate) enum ThemeMode {
    Dark,
    Light,
}

#[derive(Clone, Copy)]
/// 一套完整的界面配色。
pub(crate) struct Theme {
    pub(crate) bg: u32,
    pub(crate) sidebar: u32,
    pub(crate) card: u32,
    pub(crate) card_hover: u32,
    pub(crate) border: u32,
    pub(crate) text: u32,
    pub(crate) muted: u32,
    pub(crate) faint: u32,
    pub(crate) accent: u32,
    pub(crate) accent_soft: u32,
    pub(crate) chart_grid: u32,
    pub(crate) green: u32,
    pub(crate) yellow: u32,
    pub(crate) red: u32,
    pub(crate) peach: u32,
    pub(crate) on_accent: u32,
    pub(crate) button_hover: u32,
    pub(crate) danger_bg: u32,
    pub(crate) danger_hover: u32,
    pub(crate) success_bg: u32,
    pub(crate) success_text: u32,
    pub(crate) error_bg: u32,
    pub(crate) error_text: u32,
    pub(crate) warn_bg: u32,
    pub(crate) warn_text: u32,
}

/// 深色：近黑中性底 + 单一冷蓝强调色。
pub(crate) const DARK: Theme = Theme {
    bg: 0x0d1015,
    sidebar: 0x12161d,
    card: 0x161b23,
    card_hover: 0x1e2530,
    border: 0x262e3a,
    text: 0xe7eaf0,
    muted: 0xa2aab8,
    faint: 0x7d8798,
    accent: 0x7fa6f8,
    accent_soft: 0x1a2536,
    chart_grid: 0x222a36,
    green: 0x82c07c,
    yellow: 0xe3c176,
    red: 0xe27a72,
    peach: 0xd29a63,
    on_accent: 0x0b111c,
    button_hover: 0x232b37,
    danger_bg: 0x2b1a1d,
    danger_hover: 0x3a2327,
    success_bg: 0x16271b,
    success_text: 0x8fd08a,
    error_bg: 0x2b1a1d,
    error_text: 0xe68a83,
    warn_bg: 0x2a2418,
    warn_text: 0xe3c176,
};

/// 浅色：暖纸底 + 深铜色强调色（白字对比度 ≥ 4.5:1）。
pub(crate) const LIGHT: Theme = Theme {
    bg: 0xf6f3ec,
    sidebar: 0xefeae0,
    card: 0xfcfaf6,
    card_hover: 0xf2ede3,
    border: 0xe4ded2,
    text: 0x22262b,
    muted: 0x6d6a63,
    faint: 0x726d64,
    accent: 0x7a5a12,
    accent_soft: 0xf0e6ce,
    chart_grid: 0xe8e2d6,
    green: 0x4c7a43,
    yellow: 0xa9760f,
    red: 0xb04a43,
    peach: 0xa05f2a,
    on_accent: 0xffffff,
    button_hover: 0xe2dacb,
    danger_bg: 0xf7e7e4,
    danger_hover: 0xefd5d0,
    success_bg: 0xeaf1e4,
    success_text: 0x426b3a,
    error_bg: 0xf7e7e4,
    error_text: 0x9e3f3a,
    warn_bg: 0xf8efd8,
    warn_text: 0x7a5a12,
};
