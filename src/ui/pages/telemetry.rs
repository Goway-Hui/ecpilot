//! telemetry 页面。

use super::*;

impl AppState {
    /// 遥测页。
    pub(super) fn telemetry(&mut self, _cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
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

        let power = self.metric(
            self.t("功耗", "Power"),
            format!("{:.1}", self.status.gpu_power),
            "W".into(),
            theme.peach,
        );
        let utilization = self.metric(
            self.t("利用率", "Utilization"),
            self.status.gpu_util.to_string(),
            "%".into(),
            theme.accent,
        );
        let nv_temp = self.metric(
            self.t("NVIDIA 温度", "NVIDIA temperature"),
            self.status.gpu_temp_nv.to_string(),
            "°C".into(),
            temp_color,
        );
        let ec_temp = self.metric(
            self.t("EC 传感器", "EC sensor"),
            self.status.gpu_temp.to_string(),
            "°C".into(),
            theme.muted,
        );
        let metrics = if self.compact {
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(div().flex().gap(px(12.0)).child(power).child(utilization))
                .child(div().flex().gap(px(12.0)).child(nv_temp).child(ec_temp))
        } else {
            div()
                .flex()
                .gap(px(14.0))
                .child(power)
                .child(utilization)
                .child(nv_temp)
                .child(ec_temp)
        };

        let util = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_end()
                    .child(self.card_title("GPU 利用率", "GPU utilization"))
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(30.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(theme.text))
                                    .child(self.status.gpu_util.to_string()),
                            )
                            .child(
                                div()
                                    .pb(px(5.0))
                                    .text_size(px(12.0))
                                    .text_color(rgb(theme.muted))
                                    .child("%"),
                            ),
                    ),
            )
            .child(self.line_chart(&self.gpu_history, theme.accent, 0.0, 100.0, 120.0));

        let vram = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(self.card_title("显存", "VRAM"))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(30.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.text))
                            .child(self.status.gpu_mem_used.to_string()),
                    )
                    .child(
                        div()
                            .pb(px(5.0))
                            .text_size(px(12.0))
                            .text_color(rgb(theme.muted))
                            .child(format!("/ {} MiB", self.status.gpu_mem_total)),
                    ),
            )
            .child(self.progress(mem_percent, theme.accent))
            .child(self.hint(format!("{:.0}% {}", mem_percent, self.t("已使用", "used"))));

        let name = self
            .card()
            .flex()
            .justify_between()
            .items_center()
            .child(self.card_title("GPU", "GPU"))
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(theme.text))
                    .child(self.status.gpu_name.clone()),
            );

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(metrics)
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .items_stretch()
                    .child(util)
                    .child(vram),
            )
            .child(name)
    }
}
