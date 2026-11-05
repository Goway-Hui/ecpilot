//! 通用 UI 控件与左侧导航。
//!
//! 页面布局在 `pages` 子模块；本模块提供卡片、按钮、图表等可复用组件。

use std::collections::VecDeque;

use gpui::{
    Bounds, Context, Div, IntoElement, PathBuilder, SharedString, canvas, div, point, prelude::*,
    px, quad, rgb, size, transparent_black,
};

use crate::app::{AppState, Page};
use crate::config::{save_custom, CurveTarget};

mod pages;

impl AppState {
    /// 通用卡片容器（背景、圆角、边框）。
    pub(crate) fn card(&self) -> Div {
        div()
            .p_4()
            .rounded_lg()
            .bg(rgb(self.theme().card))
            .border_1()
            .border_color(rgb(self.theme().border))
    }

    /// 分组标题。
    pub(crate) fn section(&self, zh: &str, en: &str) -> Div {
        div()
            .px_3()
            .pt_3()
            .pb_1()
            .text_size(px(11.0))
            .text_color(rgb(self.theme().muted))
            .child(self.t(zh, en))
    }

    /// 小型柱状趋势图。
    pub(crate) fn sparkline(&self, history: &VecDeque<f32>, color: u32, columns: usize, rows: usize) -> Div {
        let theme = self.theme();
        let mut chart = div()
            .flex()
            .items_end()
            .gap(px(2.0))
            .h(px((rows as f32) * 7.0));
        let len = history.len();
        for i in 0..columns {
            let index = len.saturating_sub(columns) + i;
            let value = history
                .get(index)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, 100.0);
            let filled = ((value / 100.0) * rows as f32).ceil() as usize;
            let mut column = div().flex().flex_col().gap(px(2.0));
            for row in 0..rows {
                let on = row >= rows - filled;
                column = column.child(
                    div()
                        .size(px(5.0))
                        .rounded_md()
                        .bg(rgb(if on { color } else { theme.border })),
                );
            }
            chart = chart.child(column);
        }
        chart
    }

    #[allow(clippy::too_many_arguments)]
    /// 概览页的统计卡片（数值 + 趋势）。
    pub(crate) fn stat_card(
        &self,
        icon: &'static str,
        title: String,
        value: String,
        unit: String,
        sub: String,
        color: u32,
        history: &VecDeque<f32>,
    ) -> Div {
        let theme = self.theme();
        self.card()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .size(px(28.0))
                            .rounded_md()
                            .bg(rgb(theme.card_hover))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(color))
                            .child(icon),
                    )
                    .child(
                        div()
                            .text_color(rgb(color))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(title),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(32.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(value),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(theme.muted))
                            .child(unit),
                    ),
            )
            .child(self.label(sub))
            .child(self.sparkline(history, color, 30, 5))
    }

    /// 普通文本标签。
    pub(crate) fn label(&self, text: String) -> Div {
        div()
            .text_size(px(12.0))
            .text_color(rgb(self.theme().muted))
            .child(text)
    }

    /// 带单位和颜色的数值展示。
    pub(crate) fn metric(&self, label: String, value: String, unit: String, color: u32) -> Div {
        self.card()
            .flex()
            .flex_col()
            .gap_1()
            .flex_1()
            .child(self.label(label))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(30.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(color))
                            .child(value),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(self.theme().muted))
                            .child(unit),
                    ),
            )
    }

    /// 小徽章/状态标签。
    pub(crate) fn badge(&self, text: String, color: u32) -> Div {
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(rgb(self.theme().card_hover))
            .text_size(px(11.0))
            .text_color(rgb(color))
            .child(text)
    }

    /// 进度条。
    pub(crate) fn progress(&self, percent: f32, color: u32) -> Div {
        let width = (percent.clamp(0.0, 100.0) / 100.0) * 220.0;
        div()
            .h(px(7.0))
            .w_full()
            .rounded_full()
            .bg(rgb(self.theme().card_hover))
            .child(
                div()
                    .h_full()
                    .w(px(width.max(3.0)))
                    .rounded_full()
                    .bg(rgb(color)),
            )
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
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .bg(if active {
                rgb(theme.accent)
            } else {
                rgb(theme.sidebar)
            })
            .text_color(if active {
                rgb(theme.on_accent)
            } else {
                rgb(theme.text)
            })
            .hover(move |style| {
                style.bg(rgb(if active {
                    theme.accent
                } else {
                    theme.card_hover
                }))
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.page = page;
                cx.notify();
            }))
            .child(label)
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(if active { theme.on_accent } else { theme.muted }))
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
        div()
            .id(id)
            .px_4()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .bg(if active { rgb(self.theme().accent) } else { rgb(self.theme().card_hover) })
            .text_color(if active { rgb(self.theme().bg) } else { rgb(self.theme().text) })
            .font_weight(if active {
                gpui::FontWeight::BOLD
            } else {
                gpui::FontWeight::NORMAL
            })
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
        let (fg, bg, hover) = if danger {
            (theme.red, theme.danger_bg, theme.danger_hover)
        } else {
            (theme.text, theme.card_hover, theme.button_hover)
        };
        div()
            .id(id)
            .px_4()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .bg(rgb(bg))
            .border_1()
            .border_color(rgb(if danger { theme.red } else { theme.border }))
            .text_color(rgb(fg))
            .hover(move |style| style.bg(rgb(hover)))
            .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
            .child(label)
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
            .gap_1()
            .child(
                div()
                    .id(minus)
                    .size(px(22.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .bg(rgb(theme.card_hover))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.adjust_custom(temperature, index, -5, cx)
                    }))
                    .child("−"),
            )
            .child(
                div()
                    .w(px(28.0))
                    .text_size(px(11.0))
                    .text_color(rgb(theme.text))
                    .child(value.to_string()),
            )
            .child(
                div()
                    .id(plus)
                    .size(px(22.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .bg(rgb(theme.card_hover))
                    .cursor_pointer()
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
        let border = theme.border;
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
                    let mut grid = PathBuilder::stroke(px(1.0));
                    grid.move_to(point(bounds.left() + padding, y));
                    grid.line_to(point(bounds.right() - padding, y));
                    if let Ok(path) = grid.build() {
                        window.paint_path(path, rgb(border));
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

        let mut speeds_row = div().flex().w_full().justify_between().px_2();
        let mut temps_row = div().flex().w_full().justify_between().px_2();
        for index in 0..7 {
            speeds_row = speeds_row.child(
                div()
                    .w(px(44.0))
                    .flex()
                    .justify_center()
                    .text_size(px(10.0))
                    .text_color(rgb(theme.muted))
                    .child(format!("{}%", speeds[index])),
            );
            temps_row = temps_row.child(
                div()
                    .w(px(44.0))
                    .flex()
                    .justify_center()
                    .text_size(px(10.0))
                    .text_color(rgb(theme.muted))
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
            .gap_1()
            .child(
                div()
                    .w_full()
                    .h(px(180.0))
                    .rounded_md()
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
            .w(px(210.0))
            .h_full()
            .flex()
            .flex_col()
            .gap_1()
            .p_3()
            .bg(rgb(theme.sidebar))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .pb_3()
                    .child(
                        div()
                            .text_size(px(19.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("ECPilot"),
                    )
                    .child(self.label("MSI Alpha 17 · Linux".into())),
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
                    .text_size(px(11.0))
                    .text_color(rgb(theme.muted))
                    .child(format!("v{} · GPUI", env!("CARGO_PKG_VERSION"))),
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
            .items_center()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(24.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(self.page_title()),
                    )
                    .child(self.label(subtitle)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.badge("1s".into(), theme.muted))
                    .child(self.badge(format!("EC {}", self.status.fw), theme.accent))
                    .child(self.badge(self.status.fan_mode.clone(), theme.green))
                    .child(self.badge(self.status.shift_mode.clone(), theme.peach)),
            )
    }

    /// 系统详情中的一行。
    pub(crate) fn info_line(&self, label: &str, value: String) -> Div {
        let theme = self.theme();
        div()
            .flex()
            .gap_2()
            .py(px(1.0))
            .child(
                div()
                    .w(px(54.0))
                    .text_size(px(12.0))
                    .text_color(rgb(theme.muted))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(theme.text))
                    .child(value),
            )
    }

}
