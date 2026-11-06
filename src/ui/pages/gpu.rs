//! gpu 页面。

use super::*;

impl AppState {
    /// GPU / MUX 页。
    pub(super) fn gpu(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let discrete = self.mux == "Discrete";
        let mem_percent = if self.status.gpu_mem_total > 0 {
            self.status.gpu_mem_used as f32 / self.status.gpu_mem_total as f32 * 100.0
        } else {
            0.0
        };
        let temp_color = if self.status.gpu_temp_nv >= 80 {
            theme.red
        } else if self.status.gpu_temp_nv >= 70 {
            theme.yellow
        } else {
            theme.green
        };

        let mode = self
            .card()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(self.card_title("当前配置", "Configured mode"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(
                        div()
                            .text_size(px(30.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(if discrete {
                                theme.green
                            } else {
                                theme.accent
                            }))
                            .child(if discrete {
                                self.t("独显直连", "Discrete")
                            } else {
                                self.t("混合模式", "Hybrid")
                            }),
                    )
                    .child(self.badge(self.mux.clone(), theme.muted)),
            )
            .child(self.hint(format!(
                "{} {}",
                self.t("GPU", "GPU"),
                self.status.gpu_name
            )));

        let nvidia = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.card_title("NVIDIA", "NVIDIA"))
            .child(self.info_row(
                self.t("温度", "Temperature").as_str(),
                format!("{} °C", self.status.gpu_temp_nv),
                temp_color,
            ))
            .child(self.info_row(
                self.t("功耗", "Power").as_str(),
                format!("{:.1} W", self.status.gpu_power),
                theme.peach,
            ))
            .child(self.info_row(
                self.t("利用率", "Utilization").as_str(),
                format!("{} %", self.status.gpu_util),
                theme.accent,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .pt(px(2.0))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(self.hint(self.t("显存", "VRAM")))
                            .child(self.hint(format!(
                                "{} / {} MiB",
                                self.status.gpu_mem_used, self.status.gpu_mem_total
                            ))),
                    )
                    .child(self.progress(mem_percent, theme.accent)),
            );

        let igpu = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.card_title("AMD iGPU", "AMD iGPU"))
            .child(self.info_row(
                self.t("温度", "Temperature").as_str(),
                if self.status.igpu_temp > 0 {
                    format!("{} °C", self.status.igpu_temp)
                } else {
                    "-".into()
                },
                theme.muted,
            ))
            .child(self.info_row(
                self.t("功耗", "Power").as_str(),
                if self.status.igpu_power > 0.0 {
                    format!("{:.1} W", self.status.igpu_power)
                } else {
                    "-".into()
                },
                theme.muted,
            ))
            .child(self.hint(self.t(
                "切换后需要重启；切换期间内屏可能无输出。",
                "A reboot is required; the internal panel may go dark.",
            )));

        let (primary_label, primary_discrete) = if discrete {
            (self.t("切换到混合模式", "Switch to Hybrid"), false)
        } else {
            (self.t("切换到独显直连", "Switch to Discrete"), true)
        };
        let switch = self
            .card()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(self.card_title("切换", "Switch"))
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(self.primary_button("mux-primary", primary_label, cx, move |this, cx| {
                        this.run_action(cx, move || ec::set_mux(primary_discrete));
                    })),
            )
            .child(
                div()
                    .px(px(12.0))
                    .py(px(10.0))
                    .rounded(px(crate::theme::RADIUS_CONTROL))
                    .bg(rgb(theme.warn_bg))
                    .text_size(px(12.0))
                    .line_height(px(18.0))
                    .text_color(rgb(theme.warn_text))
                    .child(self.t(
                        "切换后需要重启。若内屏无输出，请使用 HDMI 或 SSH 执行 msi-gpu-switcher igpu 恢复。",
                        "A reboot is required. If the internal panel stays dark, use HDMI or SSH and run msi-gpu-switcher igpu.",
                    )),
            );

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(mode)
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .items_stretch()
                    .child(nvidia)
                    .child(igpu),
            )
            .child(switch)
    }
}
