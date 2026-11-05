//! telemetry 页面。

use super::*;

impl AppState {
    /// 遥测页。
    pub(super) fn telemetry(&mut self, _cx: &mut Context<Self>) -> Div {
        let mem_percent = if self.status.gpu_mem_total > 0 {
            self.status.gpu_mem_used as f32 / self.status.gpu_mem_total as f32 * 100.0
        } else {
            0.0
        };
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.metric(
                        self.t("功耗", "Power"),
                        format!("{:.1}", self.status.gpu_power),
                        "W".into(),
                        self.theme().peach,
                    ))
                    .child(self.metric(
                        self.t("利用率", "Utilization"),
                        self.status.gpu_util.to_string(),
                        "%".into(),
                        self.theme().green,
                    ))
                    .child(self.metric(
                        self.t("NVIDIA 温度", "NVIDIA temperature"),
                        self.status.gpu_temp_nv.to_string(),
                        "°C".into(),
                        if self.status.gpu_temp_nv >= 80 { self.theme().red } else { self.theme().yellow },
                    ))
                    .child(self.metric(
                        self.t("EC 传感器", "EC sensor"),
                        self.status.gpu_temp.to_string(),
                        "°C".into(),
                        self.theme().muted,
                    )),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(self.label(self.t("GPU 利用率", "GPU utilization")))
                            .child(self.label(format!("{} %", self.status.gpu_util))),
                    )
                    .child(self.progress(self.status.gpu_util as f32, self.theme().green))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(self.label(self.t("显存", "VRAM")))
                            .child(self.label(format!(
                                "{} / {} MiB",
                                self.status.gpu_mem_used, self.status.gpu_mem_total
                            ))),
                    )
                    .child(self.progress(mem_percent, self.theme().accent)),
            )
            .child(
                self.card()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(self.label(self.t("GPU", "GPU")))
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(self.status.gpu_name.clone()),
                    ),
            )
    }

}
