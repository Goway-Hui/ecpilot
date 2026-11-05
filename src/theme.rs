//! 主题与配色定义。
//!
//! 深色/浅色两套配色集中在这里，界面代码只通过 `Theme` 字段取色。

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
    pub(crate) accent: u32,
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

/// Catppuccin Mocha 风格的深色配色。
pub(crate) const DARK: Theme = Theme {
    bg: 0x11111b,
    sidebar: 0x181825,
    card: 0x1e1e2e,
    card_hover: 0x313244,
    border: 0x313244,
    text: 0xcdd6f4,
    muted: 0xa6adc8,
    accent: 0x89b4fa,
    green: 0xa6e3a1,
    yellow: 0xf9e2af,
    red: 0xf38ba8,
    peach: 0xfab387,
    on_accent: 0x11111b,
    button_hover: 0x45475a,
    danger_bg: 0x2a1a22,
    danger_hover: 0x3a2430,
    success_bg: 0x1e3a2e,
    success_text: 0xa6e3a1,
    error_bg: 0x3a1e24,
    error_text: 0xf38ba8,
    warn_bg: 0x2a2418,
    warn_text: 0xf9e2af,
};

/// 暖色纸张风格的浅色配色。
pub(crate) const LIGHT: Theme = Theme {
    bg: 0xf4f0e8,
    sidebar: 0xebe5d9,
    card: 0xfdfbf7,
    card_hover: 0xe6dfd1,
    border: 0xe2dacb,
    text: 0x3f3a33,
    muted: 0x7d7568,
    accent: 0xa8802e,
    green: 0x6f8f4f,
    yellow: 0xb8860b,
    red: 0xb85450,
    peach: 0xc07a3a,
    on_accent: 0xffffff,
    button_hover: 0xdcd4c4,
    danger_bg: 0xf6e3e1,
    danger_hover: 0xefd2cf,
    success_bg: 0xe9f0e1,
    success_text: 0x4f7038,
    error_bg: 0xf6e3e1,
    error_text: 0x9e3f3b,
    warn_bg: 0xf8eed3,
    warn_text: 0x8a6d00,
};

