//! keyboard 页面。

use super::*;

impl AppState {
    /// 键盘 RGB 页。
    pub(super) fn keyboard(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let colors = [
            ("rgb-red", (255, 0, 0)),
            ("rgb-green", (0, 255, 0)),
            ("rgb-blue", (0, 0, 255)),
            ("rgb-white", (255, 255, 255)),
            ("rgb-orange", (255, 128, 0)),
            ("rgb-purple", (180, 0, 255)),
        ];
        let detected = self
            .klc
            .as_ref()
            .map(|(path, model)| format!("{model} @ {path}"))
            .unwrap_or_else(|| self.t("未检测到 KLC 键盘", "No KLC keyboard detected"));
        let selected = self.color;
        let selected_u32 =
            ((selected.0 as u32) << 16) | ((selected.1 as u32) << 8) | selected.2 as u32;

        let mut swatches = div().flex().gap(px(8.0));
        for (id, color) in colors {
            let (r, g, b) = color;
            let is_selected = self.color == color;
            swatches = swatches.child(
                div()
                    .id(id)
                    .size(px(34.0))
                    .rounded(px(8.0))
                    .bg(rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32))
                    .border_2()
                    .border_color(rgb(if is_selected {
                        theme.accent
                    } else {
                        theme.border
                    }))
                    .cursor_pointer()
                    .hover(move |style| style.border_color(rgb(theme.accent)))
                    .active(|style| style.opacity(0.85))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = color;
                        let brightness = this.brightness;
                        this.run_action(cx, move || {
                            ec::klc_color(color.0, color.1, color.2, brightness)
                        })
                    })),
            );
        }

        let color_card = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(self.card_title("颜色", "Color"))
            .child(
                div()
                    .w_full()
                    .h(px(112.0))
                    .rounded(px(10.0))
                    .bg(rgb(selected_u32))
                    .border_1()
                    .border_color(rgb(theme.border)),
            )
            .child(swatches)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.text))
                            .child(format!(
                                "#{:02X}{:02X}{:02X}",
                                selected.0, selected.1, selected.2
                            )),
                    )
                    .child(self.hint(self.t(
                        "软件颜色，不写入键盘闪存",
                        "software color, not saved to flash",
                    ))),
            );

        let brightness_card = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(self.card_title("亮度", "Brightness"))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(34.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.text))
                            .child(self.brightness.to_string()),
                    )
                    .child(
                        div()
                            .pb(px(6.0))
                            .text_size(px(12.0))
                            .text_color(rgb(theme.muted))
                            .child("/ 255"),
                    ),
            )
            .child(self.progress(self.brightness as f32 / 2.55, theme.accent))
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(self.action_button(
                        "bright-down",
                        "− 32".into(),
                        false,
                        cx,
                        |this, cx| {
                            this.brightness = this.brightness.saturating_sub(32);
                            let (r, g, b) = this.color;
                            let brightness = this.brightness;
                            this.run_action(cx, move || ec::klc_color(r, g, b, brightness))
                        },
                    ))
                    .child(self.action_button(
                        "bright-up",
                        "+ 32".into(),
                        false,
                        cx,
                        |this, cx| {
                            this.brightness = this.brightness.saturating_add(32);
                            let (r, g, b) = this.color;
                            let brightness = this.brightness;
                            this.run_action(cx, move || ec::klc_color(r, g, b, brightness))
                        },
                    )),
            )
            .child(self.hint(self.t(
                "亮度只作用于当前颜色；切换颜色后保持不变。",
                "Brightness applies to the current color and stays when you switch.",
            )));

        let device = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.card_title("设备", "Device"))
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(theme.text))
                    .child(detected),
            )
            .child(self.hint(self.t(
                "通过 pkexec 提权写入 hidraw；颜色仅保存在内存。",
                "Writes go through pkexec to hidraw; colors live in RAM only.",
            )));

        let off = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.card_title("关闭灯光", "Lights off"))
            .child(self.hint(self.t(
                "关闭全部按键背光，保留当前颜色选择。",
                "Turns every key off and keeps the current color choice.",
            )))
            .child(
                div().flex().justify_end().child(self.action_button(
                    "rgb-off",
                    self.t("关闭全部灯光", "All lights off"),
                    true,
                    cx,
                    |this, cx| this.run_action(cx, ec::klc_off),
                )),
            );

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .items_stretch()
                    .child(color_card)
                    .child(brightness_card),
            )
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .items_stretch()
                    .child(device)
                    .child(off),
            )
    }
}
