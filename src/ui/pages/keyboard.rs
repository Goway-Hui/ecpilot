//! keyboard 页面。

use super::*;

impl AppState {
    /// 键盘 RGB 页。
    pub(super) fn keyboard(&mut self, cx: &mut Context<Self>) -> Div {
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

        let mut swatches = div().flex().gap_2();
        for (id, color) in colors {
            let (r, g, b) = color;
            let selected = self.color == color;
            swatches = swatches.child(
                div()
                    .id(id)
                    .size(px(34.0))
                    .rounded_md()
                    .bg(rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32))
                    .border_2()
                    .border_color(rgb(if selected { self.theme().accent } else { self.theme().border }))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color = color;
                        this.action(ec::klc_color(color.0, color.1, color.2, this.brightness), cx)
                    })),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.label(self.t("设备", "Device")))
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(detected),
                    )
                    .child(self.label(self.t(
                        "写入通过 pkexec 提权；颜色为 RAM 级，不写闪存。",
                        "Writes go through pkexec; colors are RAM-only.",
                    ))),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("颜色", "Color")))
                    .child(swatches)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size(px(26.0))
                                    .rounded_md()
                                    .bg(rgb(((self.color.0 as u32) << 16)
                                        | ((self.color.1 as u32) << 8)
                                        | self.color.2 as u32))
                                    .border_1()
                                    .border_color(rgb(self.theme().border)),
                            )
                            .child(self.label(format!(
                                "#{:02X}{:02X}{:02X}",
                                self.color.0, self.color.1, self.color.2
                            ))),
                    ),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("亮度", "Brightness")))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(self.action_button("bright-down", "−".into(), false, cx, |this, cx| {
                                this.brightness = this.brightness.saturating_sub(32);
                                let (r, g, b) = this.color;
                                this.action(ec::klc_color(r, g, b, this.brightness), cx)
                            }))
                            .child(
                                div()
                                    .text_size(px(22.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child(format!("{} / 255", self.brightness)),
                            )
                            .child(self.action_button("bright-up", "+".into(), false, cx, |this, cx| {
                                this.brightness = this.brightness.saturating_add(32);
                                let (r, g, b) = this.color;
                                this.action(ec::klc_color(r, g, b, this.brightness), cx)
                            })),
                    )
                    .child(self.progress(self.brightness as f32 / 2.55, self.theme().accent)),
            )
            .child(
                div().flex().justify_end().child(self.action_button(
                    "rgb-off",
                    self.t("关闭全部灯光", "All lights off"),
                    true,
                    cx,
                    |this, cx| this.action(ec::klc_off(), cx),
                )),
            )
    }

}
