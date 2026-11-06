//! 通用 UI 控件与左侧导航。
//!
//! 页面布局在 `pages` 子模块；本模块提供卡片、按钮、图表等可复用组件。
//! 设计约束：一个间距节奏、一套字号、一个圆角族、一个强调色。

use std::collections::VecDeque;

use gpui::{
    Bounds, Context, Div, IntoElement, PathBuilder, SharedString, canvas, div, point, prelude::*,
    px, quad, relative, rgb, size, transparent_black,
};

use crate::app::{AppState, Page};
use crate::config::{save_custom, CurveTarget};
use crate::theme::{RADIUS_CARD, RADIUS_CONTROL};

mod pages;

/// 常用字号（桌面端密度，正文不小于 13px）。
const T_LABEL: f32 = 12.5;
const T_BODY: f32 = 13.0;
const T_VALUE: f32 = 34.0;

impl AppState {
    /// 通用卡片容器（背景、圆角、边框）。
    pub(crate) fn card(&self) -> Div {
        let theme = self.theme();
        div()
            .min_w(px(0.0))
            .p(px(20.0))
            .rounded(px(RADIUS_CARD))
            .bg(rgb(theme.card))
            .border_1()
            .border_color(rgb(theme.border))
    }

    /// 卡片/分组标题。
    pub(crate) fn section(&self, zh: &str, en: &str) -> Div {
        div()
            .px(px(10.0))
            .pt(px(14.0))
            .pb(px(6.0))
            .text_size(px(12.0))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(self.theme().faint))
            .child(self.t(zh, en))
    }

    /// 页内分组标题（与卡片左边缘对齐）。
    pub(crate) fn page_section(&self, zh: &str, en: &str) -> Div {
        div()
            .pt(px(6.0))
            .pb(px(2.0))
            .text_size(px(12.5))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(self.theme().faint))
            .child(self.t(zh, en))
    }

    /// 卡片内的小标题。
    pub(crate) fn card_title(&self, zh: &str, en: &str) -> Div {
        div()
            .text_size(px(T_LABEL))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(self.theme().muted))
            .child(self.t(zh, en))
    }

    /// 普通文本标签。
    pub(crate) fn label(&self, text: String) -> Div {
        div()
            .text_size(px(T_BODY))
            .line_height(px(19.0))
            .whitespace_normal()
            .text_color(rgb(self.theme().muted))
            .child(text)
    }

    /// 更弱的说明文字。
    pub(crate) fn hint(&self, text: String) -> Div {
        div()
            .text_size(px(12.0))
            .line_height(px(17.0))
            .whitespace_normal()
            .text_color(rgb(self.theme().faint))
            .child(text)
    }

    /// 系统详情中的一行。
    pub(crate) fn info_line(&self, label: &str, value: String) -> Div {
        let theme = self.theme();
        div()
            .flex()
            .gap(px(10.0))
            .py(px(2.0))
            .child(
                div()
                    .w(px(64.0))
                    .text_size(px(T_LABEL))
                    .text_color(rgb(theme.faint))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .whitespace_normal()
                    .text_size(px(T_BODY))
                    .text_color(rgb(theme.text))
                    .child(value),
            )
    }

    /// 模式/状态卡中的一行（左右分布）。
    pub(crate) fn info_row(&self, label: &str, value: String, color: u32) -> Div {
        let theme = self.theme();
        div()
            .flex()
            .items_center()
            .justify_between()
            .py(px(5.0))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .whitespace_normal()
                    .text_size(px(T_BODY))
                    .text_color(rgb(theme.muted))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .flex()
                    .min_w(px(0.0))
                    .items_center()
                    .gap(px(7.0))
                    .child(div().size(px(6.0)).rounded_full().bg(rgb(color)))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .whitespace_normal()
                            .text_size(px(T_BODY))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.text))
                            .child(value),
                    ),
            )
    }

    /// 带单位和颜色的数值展示。
    pub(crate) fn metric(&self, label: String, value: String, unit: String, color: u32) -> Div {
        let theme = self.theme();
        self.card()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .flex_1()
            .child(self.label(label))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(30.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(color))
                            .child(value),
                    )
                    .child(
                        div()
                            .pb(px(4.0))
                            .text_size(px(12.0))
                            .text_color(rgb(theme.muted))
                            .child(unit),
                    ),
            )
    }

    /// 小徽章/状态标签。
    pub(crate) fn badge(&self, text: String, color: u32) -> Div {
        div()
            .px(px(8.0))
            .py(px(3.0))
            .rounded(px(6.0))
            .bg(rgb(self.theme().card_hover))
            .text_size(px(12.0))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(color))
            .child(text)
    }

    /// 进度条。
    pub(crate) fn progress(&self, percent: f32, color: u32) -> Div {
        let width = percent.clamp(0.0, 100.0) / 100.0;
        div()
            .h(px(6.0))
            .w_full()
            .rounded_full()
            .bg(rgb(self.theme().card_hover))
            .child(
                div()
                    .h_full()
                    .w(relative(width.max(0.02)))
                    .rounded_full()
                    .bg(rgb(color)),
            )
    }

    /// 顶部状态栏里的一个点+文字。
    pub(crate) fn status_item(&self, value: String, color: u32) -> Div {
        let theme = self.theme();
        div()
            .flex()
            .items_center()
            .gap(px(7.0))
            .child(div().size(px(6.0)).rounded_full().bg(rgb(color)))
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(theme.muted))
                    .child(value),
            )
    }

    /// 一个紧凑的统计卡：标题、数值、副行、趋势线。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stat_card(
        &self,
        title: String,
        value: String,
        unit: String,
        sub: String,
        color: u32,
        history: &VecDeque<f32>,
        min: f32,
        max: f32,
    ) -> Div {
        let theme = self.theme();
        self.card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .child(div().size(px(7.0)).rounded_full().bg(rgb(color)))
                    .child(
                        div()
                            .text_size(px(T_LABEL))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.muted))
                            .child(title),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(T_VALUE))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.text))
                            .child(value),
                    )
                    .child(
                        div()
                            .pb(px(6.0))
                            .text_size(px(12.0))
                            .text_color(rgb(theme.muted))
                            .child(unit),
                    ),
            )
            .child(self.hint(sub))
            .child(self.line_chart(history, color, min, max, 42.0))
    }

    /// 折线趋势图；无交互，仅表达最近 48 个采样。
    pub(crate) fn line_chart(
        &self,
        history: &VecDeque<f32>,
        color: u32,
        min: f32,
        max: f32,
        height: f32,
    ) -> Div {
        let grid = self.theme().chart_grid;
        let samples: Vec<f32> = history.iter().copied().collect();
        let span = (max - min).max(0.001);
        let chart = canvas(
            move |_, _, _| {},
            move |bounds, _, window, _| {
                let pad = px(2.0);
                let width = bounds.size.width - pad - pad;
                let h = bounds.size.height - pad - pad;
                let y_at = |value: f32| {
                    bounds.origin.y + pad
                        + h * (1.0 - ((value - min) / span).clamp(0.0, 1.0))
                };

                for level in [0.25f32, 0.5, 0.75] {
                    let y = bounds.origin.y + pad + h * level;
                    let mut line = PathBuilder::stroke(px(1.0));
                    line.move_to(point(bounds.origin.x + pad, y));
                    line.line_to(point(bounds.origin.x + pad + width, y));
                    if let Ok(path) = line.build() {
                        window.paint_path(path, rgb(grid));
                    }
                }

                let count = samples.len();
                if count < 2 {
                    return;
                }
                let mut line = PathBuilder::stroke(px(2.0));
                for (index, &value) in samples.iter().enumerate() {
                    let x = bounds.origin.x + pad
                        + width * (index as f32 / (count.saturating_sub(1)) as f32);
                    let position = point(x, y_at(value));
                    if index == 0 {
                        line.move_to(position);
                    } else {
                        line.line_to(position);
                    }
                }
                if let Ok(path) = line.build() {
                    window.paint_path(path, rgb(color));
                }

                if let Some(&last) = samples.last() {
                    let x = bounds.origin.x + pad + width;
                    let y = y_at(last);
                    let dot = Bounds {
                        origin: point(x - px(3.0), y - px(3.0)),
                        size: size(px(6.0), px(6.0)),
                    };
                    window.paint_quad(quad(
                        dot,
                        px(3.0),
                        rgb(color),
                        px(0.0),
                        transparent_black(),
                        Default::default(),
                    ));
                }
            },
        )
        .size_full();

        div().w_full().h(px(height)).child(chart)
    }

    /// 4 段档位条。
    pub(crate) fn level_bar(&self, level: i32, active_color: u32) -> Div {
        let theme = self.theme();
        let mut row = div().flex().gap(px(4.0)).w_full();
        for index in 0..4 {
            let color = if index <= level {
                active_color
            } else {
                theme.card_hover
            };
            row = row.child(
                div()
                    .h(px(6.0))
                    .flex_1()
                    .rounded_full()
                    .bg(rgb(color)),
            );
        }
        row
    }

    /// 侧栏导航项。
    pub(crate) fn nav(
        &self,
        id: &'static str,
        page: Page,
        zh: &str,
        en: &str,
        value: Option<String>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.page == page;
        let label = self.t(zh, en);
        let theme = self.theme();
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(10.0))
            .h(px(38.0))
            .px(px(10.0))
            .rounded(px(RADIUS_CONTROL))
            .cursor_pointer()
            .bg(rgb(if active {
                theme.accent_soft
            } else {
                theme.sidebar
            }))
            .hover(move |style| {
                style.bg(rgb(if active {
                    theme.accent_soft
                } else {
                    theme.button_hover
                }))
            })
            .active(|style| style.opacity(0.86))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_page(page);
                cx.notify();
            }))
            .child(
                div()
                    .w(px(2.0))
                    .h(px(16.0))
                    .rounded_full()
                    .bg(rgb(if active { theme.accent } else { theme.sidebar })),
            )
            .child(
                div()
                    .text_size(px(T_BODY))
                    .font_weight(if active {
                        gpui::FontWeight::MEDIUM
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .text_color(rgb(if active { theme.accent } else { theme.text }))
                    .child(label),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(if active { theme.accent } else { theme.faint }))
                    .child(value.unwrap_or_default()),
            )
    }

    /// 分段选择按钮（模式、预设等）。
    pub(crate) fn pill(
        &self,
        id: &'static str,
        label: String,
        active: bool,
        cx: &mut Context<Self>,
        handler: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        div()
            .id(id)
            .h(px(34.0))
            .px(px(14.0))
            .flex()
            .items_center()
            .rounded(px(RADIUS_CONTROL))
            .cursor_pointer()
            .bg(rgb(if active {
                theme.accent
            } else {
                theme.card_hover
            }))
            .text_color(rgb(if active {
                theme.on_accent
            } else {
                theme.text
            }))
            .text_size(px(T_LABEL))
            .font_weight(gpui::FontWeight::MEDIUM)
            .hover(move |style| {
                style.bg(rgb(if active {
                    theme.accent
                } else {
                    theme.button_hover
                }))
            })
            .active(|style| style.opacity(0.88))
            .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
            .child(label)
    }

    /// 执行特权操作的按钮。
    pub(crate) fn action_button(
        &self,
        id: &'static str,
        label: String,
        danger: bool,
        cx: &mut Context<Self>,
        handler: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        let (fg, bg, hover, border) = if danger {
            (
                theme.red,
                theme.danger_bg,
                theme.danger_hover,
                theme.red,
            )
        } else {
            (theme.text, theme.card, theme.button_hover, theme.border)
        };
        div()
            .id(id)
            .h(px(34.0))
            .px(px(14.0))
            .flex()
            .items_center()
            .rounded(px(RADIUS_CONTROL))
            .cursor_pointer()
            .bg(rgb(bg))
            .border_1()
            .border_color(rgb(border))
            .text_size(px(T_LABEL))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(fg))
            .hover(move |style| style.bg(rgb(hover)))
            .active(|style| style.opacity(0.86))
            .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
            .child(label)
    }

    /// 页面唯一的主操作按钮（实心强调色）。
    pub(crate) fn primary_button(
        &self,
        id: &'static str,
        label: String,
        cx: &mut Context<Self>,
        handler: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        div()
            .id(id)
            .h(px(34.0))
            .px(px(14.0))
            .flex()
            .items_center()
            .rounded(px(RADIUS_CONTROL))
            .cursor_pointer()
            .bg(rgb(theme.accent))
            .border_1()
            .border_color(rgb(theme.accent))
            .text_size(px(T_LABEL))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(theme.on_accent))
            .hover(move |style| style.bg(rgb(theme.accent)).opacity(0.90))
            .active(|style| style.opacity(0.82))
            .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
            .child(label)
    }

    /// 开关控件：42x24 轨道 + 18px 滑块。
    pub(crate) fn switch(
        &self,
        id: &'static str,
        on: bool,
        cx: &mut Context<Self>,
        handler: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        let mut track = div()
            .id(id)
            .h(px(24.0))
            .w(px(42.0))
            .rounded_full()
            .cursor_pointer()
            .border_1()
            .bg(rgb(if on { theme.accent } else { theme.card_hover }))
            .border_color(rgb(if on { theme.accent } else { theme.border }))
            .flex()
            .items_center()
            .px(px(3.0))
            .hover(move |style| style.opacity(0.9))
            .active(|style| style.opacity(0.8))
            .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)));
        track = if on {
            track.justify_end()
        } else {
            track.justify_start()
        };
        track.child(
            div()
                .size(px(18.0))
                .rounded_full()
                .bg(rgb(theme.card))
                .border_1()
                .border_color(rgb(theme.border)),
        )
    }

    /// 调整自定义曲线的单个点。
    pub(crate) fn adjust_custom(
        &mut self,
        temperature: bool,
        index: usize,
        delta: i16,
        cx: &mut Context<Self>,
    ) {
        let (temps, speeds) = match self.curve_target {
            CurveTarget::Cpu => (&mut self.custom.cpu_temp, &mut self.custom.cpu_speed),
            CurveTarget::Gpu => (&mut self.custom.gpu_temp, &mut self.custom.gpu_speed),
        };
        if temperature {
            let lower = if index == 0 { 40 } else { temps[index - 1] + 1 };
            let upper = if index + 1 == temps.len() {
                95
            } else {
                temps[index + 1] - 1
            };
            let value = (temps[index] as i16 + delta).clamp(lower as i16, upper as i16);
            temps[index] = value as u8;
        } else {
            let mut lower = if index == 0 { 0 } else { speeds[index - 1] };
            if index + 1 == speeds.len() {
                lower = lower.max(70);
            }
            let upper = if index + 1 == speeds.len() {
                150
            } else {
                speeds[index + 1]
            };
            let value = (speeds[index] as i16 + delta).clamp(lower as i16, upper as i16);
            speeds[index] = value as u8;
        }
        save_custom(&self.custom);
        cx.notify();
    }

    /// 曲线点的加减控件。
    pub(crate) fn stepper(
        &self,
        prefix: &'static str,
        index: usize,
        value: u8,
        temperature: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = self.theme();
        let minus: SharedString = format!("{prefix}-{index}-minus").into();
        let plus: SharedString = format!("{prefix}-{index}-plus").into();
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(
                div()
                    .id(minus)
                    .size(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .bg(rgb(theme.card_hover))
                    .cursor_pointer()
                    .hover(move |style| style.bg(rgb(theme.button_hover)))
                    .active(|style| style.opacity(0.8))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.adjust_custom(temperature, index, -5, cx)
                    }))
                    .child("−"),
            )
            .child(
                div()
                    .w(px(30.0))
                    .text_size(px(12.0))
                    .text_color(rgb(theme.text))
                    .child(value.to_string()),
            )
            .child(
                div()
                    .id(plus)
                    .size(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .bg(rgb(theme.card_hover))
                    .cursor_pointer()
                    .hover(move |style| style.bg(rgb(theme.button_hover)))
                    .active(|style| style.opacity(0.8))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.adjust_custom(temperature, index, 5, cx)
                    }))
                    .child("+"),
            )
    }

    /// 风扇曲线折线图。
    pub(crate) fn curve_chart(&self, temps: &[u8; 6], speeds: &[u8; 7]) -> Div {
        let theme = self.theme();
        let speeds = *speeds;
        let accent = theme.accent;
        let grid = theme.chart_grid;
        let peach = theme.peach;
        let chart = canvas(
            move |_, _, _| {},
            move |bounds, _, window, _| {
                let padding = px(14.0);
                let width = bounds.size.width - padding - padding;
                let height = bounds.size.height - padding - padding;
                let x_at = |index: usize| bounds.left() + padding + width * (index as f32 / 6.0);
                let y_at = |value: f32| {
                    bounds.origin.y + padding + height * (1.0 - value / 150.0)
                };

                for level in [0.0f32, 50.0, 100.0, 150.0] {
                    let y = y_at(level);
                    let mut line = PathBuilder::stroke(px(1.0));
                    line.move_to(point(bounds.left() + padding, y));
                    line.line_to(point(bounds.right() - padding, y));
                    if let Ok(path) = line.build() {
                        window.paint_path(path, rgb(grid));
                    }
                }

                let mut line = PathBuilder::stroke(px(2.5));
                for (index, &speed) in speeds.iter().enumerate() {
                    let position = point(x_at(index), y_at(speed as f32));
                    if index == 0 {
                        line.move_to(position);
                    } else {
                        line.line_to(position);
                    }
                }
                if let Ok(path) = line.build() {
                    window.paint_path(path, rgb(accent));
                }

                for (index, &speed) in speeds.iter().enumerate() {
                    let center = point(x_at(index), y_at(speed as f32));
                    let dot = Bounds {
                        origin: point(center.x - px(4.0), center.y - px(4.0)),
                        size: size(px(8.0), px(8.0)),
                    };
                    window.paint_quad(quad(
                        dot,
                        px(4.0),
                        rgb(if index == 6 { peach } else { accent }),
                        px(0.0),
                        transparent_black(),
                        Default::default(),
                    ));
                }
            },
        )
        .size_full();

        let mut speeds_row = div().flex().w_full().justify_between().px(px(8.0));
        let mut temps_row = div().flex().w_full().justify_between().px(px(8.0));
        for index in 0..7 {
            speeds_row = speeds_row.child(
                div()
                    .w(px(44.0))
                    .flex()
                    .justify_center()
                    .text_size(px(12.0))
                    .text_color(rgb(theme.faint))
                    .child(format!("{}%", speeds[index])),
            );
            temps_row = temps_row.child(
                div()
                    .w(px(44.0))
                    .flex()
                    .justify_center()
                    .text_size(px(12.0))
                    .text_color(rgb(theme.faint))
                    .child(if index == 0 {
                        "—".to_string()
                    } else {
                        format!("{}°C", temps[index - 1])
                    }),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .w_full()
                    .h(px(180.0))
                    .rounded(px(RADIUS_CONTROL))
                    .bg(rgb(theme.bg))
                    .child(chart),
            )
            .child(speeds_row)
            .child(temps_row)
    }

    /// 当前页面标题。
    pub(crate) fn page_title(&self) -> String {
        match self.page {
            Page::Overview => self.t("概览", "Overview"),
            Page::Fan => self.t("风扇控制", "Fan control"),
            Page::Keyboard => self.t("键盘灯效", "Keyboard lighting"),
            Page::Gpu => self.t("GPU 与 MUX", "GPU & MUX"),
            Page::Telemetry => self.t("遥测", "Telemetry"),
            Page::Thermal => self.t("温控保护", "Thermal guard"),
            Page::Settings => self.t("设置", "Settings"),
        }
    }

    /// 当前页面副标题。
    pub(crate) fn page_subtitle(&self) -> String {
        match self.page {
            Page::Overview => self.t("实时硬件状态", "Live hardware status"),
            Page::Fan => self.t("模式与曲线预设", "Modes and curve presets"),
            Page::Keyboard => self.t("SteelSeries KLC 单键 RGB", "SteelSeries KLC per-key RGB"),
            Page::Gpu => self.t("混合与独显直连", "Hybrid and discrete graphics"),
            Page::Telemetry => self.t("NVIDIA 功耗与利用率", "NVIDIA power and utilization"),
            Page::Thermal => self.t("多级限频 · 防止热断电", "Multi-level CPU cap · thermal protection"),
            Page::Settings => self.t("语言与关于", "Language and about"),
        }
    }

    /// 左侧导航栏。
    pub(crate) fn sidebar(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        div()
            .w(px(224.0))
            .h_full()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .p(px(16.0))
            .bg(rgb(theme.sidebar))
            .border_r_1()
            .border_color(rgb(theme.border))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(11.0))
                    .px(px(8.0))
                    .py(px(10.0))
                    .pb(px(18.0))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded(px(10.0))
                            .bg(rgb(theme.accent_soft))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.5))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.accent))
                            .child("EC"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(18.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child("ECPilot"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(rgb(theme.muted))
                                    .child("MSI Alpha 17 · Linux"),
                            ),
                    ),
            )
            .child(self.section("硬件", "Hardware"))
            .child(self.nav(
                "nav-overview",
                Page::Overview,
                "概览",
                "Overview",
                Some(format!("{} °C", self.status.cpu_temp)),
                cx,
            ))
            .child(self.nav(
                "nav-fan",
                Page::Fan,
                "风扇控制",
                "Fan control",
                Some(format!("{} RPM", self.status.cpu_fan)),
                cx,
            ))
            .child(self.nav("nav-keyboard", Page::Keyboard, "键盘", "Keyboard", None, cx))
            .child(self.nav("nav-gpu", Page::Gpu, "GPU", "GPU", None, cx))
            .child(self.nav(
                "nav-telemetry",
                Page::Telemetry,
                "遥测",
                "Telemetry",
                Some(format!("{} %", self.status.gpu_util)),
                cx,
            ))
            .child(self.nav(
                "nav-thermal",
                Page::Thermal,
                "温控",
                "Thermal",
                Some(format!("{} °C", self.status.tctl)),
                cx,
            ))
            .child(self.section("系统", "System"))
            .child(self.nav("nav-settings", Page::Settings, "设置", "Settings", None, cx))
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .child(
                        div()
                            .size(px(6.0))
                            .rounded_full()
                            .bg(rgb(if self.status.thermal_active {
                                theme.green
                            } else {
                                theme.faint
                            })),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(theme.faint))
                            .child(format!(
                                "v{} · {}",
                                env!("CARGO_PKG_VERSION"),
                                if !self.status.thermal_active {
                                    self.t("温控已停止", "guard off")
                                } else if self.status.thermal_manual {
                                    self.t("温控手动", "guard manual")
                                } else {
                                    self.t("温控运行中", "guard auto")
                                }
                            )),
                    ),
            )
    }

    /// 顶部状态栏。
    pub(crate) fn header(&self) -> Div {
        let theme = self.theme();
        let subtitle = if self.page == Page::Overview {
            format!(
                "MSI Alpha 17 C7VF · EC {} · CPU {} °C / GPU {} °C (NV) · EC {} °C",
                self.status.fw, self.status.cpu_temp, self.status.gpu_temp_nv, self.status.gpu_temp
            )
        } else {
            self.page_subtitle()
        };
        div()
            .flex()
            .justify_between()
            .items_start()
            .w_full()
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(24.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.text))
                            .child(self.page_title()),
                    )
                    .child(
                        div()
                            .whitespace_normal()
                            .text_size(px(12.0))
                            .text_color(rgb(theme.muted))
                            .child(subtitle),
                    ),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .pt(px(4.0))
                    .child(self.status_item(self.status.fan_mode.clone(), theme.green))
                    .child(self.status_item(self.status.shift_mode.clone(), theme.peach)),
            )
    }

}
