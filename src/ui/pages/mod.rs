//! 应用各页面。
//!
//! 每个页面一个子模块，共享 `ui` 的通用控件与主题。

use gpui::{AnyElement, Context, Div, div, prelude::*, px, rgb};

use crate::app::{AppState, Page};
use crate::config::{
    CurveTarget, CustomCurve, Lang, save_close_to_tray, save_config, save_custom, save_fan_mode,
    save_fan_preset, save_shift_mode, save_thermal_notify,
};
use crate::ec;
use crate::thermal_config::ThermalConfig;
use crate::theme::{RADIUS_CONTROL, ThemeMode};

mod fan;
mod gpu;
mod keyboard;
mod overview;
mod settings;
mod telemetry;
mod thermal;

impl AppState {
    /// 页面主体：顶部栏 + 当前页面 + 操作提示。
    pub(crate) fn content(&mut self, cx: &mut Context<Self>) -> Div {
        // 托盘重建窗口时恢复最近使用的页面。
        crate::app::remember_page(self.page);
        let page = match self.page {
            Page::Overview => self.overview(cx).into_any_element(),
            Page::Fan => self.fan(cx).into_any_element(),
            Page::Keyboard => self.keyboard(cx).into_any_element(),
            Page::Gpu => self.gpu(cx).into_any_element(),
            Page::Telemetry => self.telemetry(cx).into_any_element(),
            Page::Thermal => self.thermal(cx).into_any_element(),
            Page::Settings => self.settings(cx).into_any_element(),
        };
        let toast: AnyElement = if self.message.is_empty() {
            div().into_any_element()
        } else {
            div()
                .px(px(12.0))
                .py(px(8.0))
                .rounded(px(RADIUS_CONTROL))
                .bg(rgb(if self.message_ok {
                    self.theme().success_bg
                } else {
                    self.theme().error_bg
                }))
                .text_color(rgb(if self.message_ok {
                    self.theme().success_text
                } else {
                    self.theme().error_text
                }))
                .text_size(px(12.5))
                .child(self.message.clone())
                .into_any_element()
        };

        div()
            .flex_1()
            .h_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .child(div().w_full().min_w(px(0.0)).px(px(28.0)).pt(px(24.0)).pb(px(10.0)).child(self.header()))
            .child(
                div()
                    .id("page-scroll")
                    .flex_1()
                    .w_full()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .overflow_y_scroll()
                    .track_scroll(&self.page_scroll)
                    .px(px(28.0))
                    .pt(px(10.0))
                    .pb(px(24.0))
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(div().w_full().min_w(px(0.0)).child(page))
                    .child(toast),
            )
    }
}
