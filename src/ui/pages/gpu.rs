//! gpu 页面。

use super::*;

impl AppState {
    /// GPU / MUX 页。
    pub(super) fn gpu(&mut self, cx: &mut Context<Self>) -> Div {
        let discrete = self.mux == "Discrete";
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("当前配置", "Configured mode")))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(28.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(if discrete { self.theme().green } else { self.theme().accent }))
                                    .child(if discrete {
                                        self.t("独显直连", "Discrete")
                                    } else {
                                        self.t("混合模式", "Hybrid")
                                    }),
                            )
                            .child(self.badge(self.mux.clone(), self.theme().muted)),
                    )
                    .child(self.label(format!(
                        "{} {}",
                        self.t("GPU", "GPU"),
                        self.status.gpu_name
                    ))),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.label("AMD iGPU".into()))
                    .child(self.info_line(
                        self.t("温度", "Temperature").as_str(),
                        if self.status.igpu_temp > 0 {
                            format!("{} °C", self.status.igpu_temp)
                        } else {
                            "-".into()
                        },
                    ))
                    .child(self.info_line(
                        self.t("功耗", "Power").as_str(),
                        if self.status.igpu_power > 0.0 {
                            format!("{:.1} W", self.status.igpu_power)
                        } else {
                            "-".into()
                        },
                    )),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("切换", "Switch")))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.action_button(
                                "mux-dgpu",
                                self.t("切换到独显直连", "Switch to Discrete"),
                                false,
                                cx,
                                |this, cx| this.action(ec::set_mux(true), cx),
                            ))
                            .child(self.action_button(
                                "mux-igpu",
                                self.t("切换到混合模式", "Switch to Hybrid"),
                                false,
                                cx,
                                |this, cx| this.action(ec::set_mux(false), cx),
                            )),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(self.theme().warn_bg))
                            .text_size(px(12.0))
                            .text_color(rgb(self.theme().warn_text))
                            .child(self.t(
                                "切换后需要重启。若内屏无输出，请使用 HDMI 或 SSH 执行 msi-gpu-switcher igpu 恢复。",
                                "A reboot is required. If the internal panel stays dark, use HDMI or SSH and run msi-gpu-switcher igpu.",
                            )),
                    ),
            )
    }

}
