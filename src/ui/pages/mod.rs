//! 应用各页面。
//!
//! 每个页面一个子模块，共享 `ui` 的通用控件与主题。

use gpui::{AnyElement, Context, Div, IntoElement, div, prelude::*, px, rgb};

use crate::app::{AppState, Page};
use crate::config::{save_close_to_tray, save_config, save_custom, save_fan_mode, save_fan_preset, save_shift_mode, CurveTarget, CustomCurve, Lang};
use crate::ec;
use crate::theme::ThemeMode;

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
                .px_3()
                .py_2()
                .rounded_md()
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
                .text_size(px(12.0))
                .child(self.message.clone())
                .into_any_element()
        };

        div()
            .flex_1()
            .h_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.header())
            .child(page)
            .child(div().flex_1())
            .child(toast)
    }
}
