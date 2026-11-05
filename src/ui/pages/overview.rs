//! overview 页面。

use super::*;

impl AppState {
    /// 概览页。
    pub(super) fn overview(&mut self, _cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let temp_color = |value: i32| {
            if value >= 85 {
                theme.red
            } else if value >= 70 {
                theme.yellow
            } else {
                theme.green
            }
        };
        let details = &self.details;
        let mut left = div().flex_1().flex().flex_col().gap_1();
        left = left
            .child(self.info_line("OS", details.os.clone()))
            .child(self.info_line(self.t("内核", "Kernel").as_str(), details.kernel.clone()))
            .child(self.info_line(self.t("包数", "Packages").as_str(), details.packages.clone()))
            .child(self.info_line(self.t("系统年龄", "Age").as_str(), details.age.clone()))
            .child(self.info_line(self.t("用户", "User").as_str(), details.user.clone()))
            .child(self.info_line("PC", details.pc.clone()))
            .child(self.info_line("WM", details.wm.clone()))
            .child(self.info_line("Shell", details.shell.clone()))
            .child(self.info_line(self.t("终端", "Terminal").as_str(), details.term.clone()))
            .child(self.info_line(self.t("字体", "Font").as_str(), details.font.clone()));

        let mut right = div().flex_1().flex().flex_col().gap_1();
        right = right
            .child(self.info_line("CPU", details.cpu.clone()))
            .child(self.info_line(self.t("内存", "Memory").as_str(), details.mem.clone()))
            .child(self.info_line(self.t("交换", "Swap").as_str(), details.swp.clone()))
            .child(self.info_line(
                "GPU",
                format!("{} {:.1}°C", self.status.gpu_name, self.status.gpu_temp_nv as f32),
            ))
            .child(self.info_line("GPU", details.gpu_amd.clone()))
            .child(self.info_line(
                self.t("风扇", "Fans").as_str(),
                format!("{} / {} RPM", self.status.cpu_fan, self.status.gpu_fan),
            ))
            .child(self.info_line(self.t("显示器", "Monitor").as_str(), details.mon.clone()));
        for disk in &details.disks {
            right = right.child(self.info_line(self.t("磁盘", "Disk").as_str(), disk.clone()));
        }

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.stat_card(
                        "▦",
                        self.t("CPU", "CPU"),
                        self.status.cpu_temp.to_string(),
                        "°C".into(),
                        format!(
                            "{} {} · {} {}",
                            self.t("风扇", "Fan"),
                            self.status.cpu_fan,
                            self.t("性能", "Shift"),
                            self.status.shift_mode
                        ),
                        temp_color(self.status.cpu_temp),
                        &self.cpu_history,
                    ))
                    .child(self.stat_card(
                        "◈",
                        self.t("GPU", "GPU"),
                        self.status.gpu_util.to_string(),
                        "%".into(),
                        format!(
                            "NV {} °C · EC {} °C · {} MiB",
                            self.status.gpu_temp_nv, self.status.gpu_temp, self.status.gpu_mem_used
                        ),
                        theme.peach,
                        &self.gpu_history,
                    )),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.stat_card(
                        "❄",
                        self.t("风扇", "Fans"),
                        self.status.cpu_fan.to_string(),
                        "RPM".into(),
                        format!("GPU {} RPM", self.status.gpu_fan),
                        theme.accent,
                        &self.fan_history,
                    ))
                    .child(self.stat_card(
                        "▮",
                        self.t("电池", "Battery"),
                        self.status.battery.to_string(),
                        "%".into(),
                        format!(
                            "{} · {} {}%",
                            self.status.charging,
                            self.t("上限", "limit"),
                            self.status.threshold
                        ),
                        theme.green,
                        &self.battery_history,
                    ))
                    .child(
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
                                            .text_color(rgb(theme.peach))
                                            .child("⚙"),
                                    )
                                    .child(
                                        div()
                                            .text_color(rgb(theme.peach))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .child(self.t("模式", "Mode")),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(24.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child(self.status.fan_mode.clone()),
                            )
                            .child(self.label(format!(
                                "{} {}",
                                self.t("性能", "Shift"),
                                self.status.shift_mode
                            )))
                            .child(self.badge(
                                format!(
                                    "Cooler Boost {}",
                                    if self.status.cooler_boost {
                                        self.t("开启", "on")
                                    } else {
                                        self.t("关闭", "off")
                                    }
                                ),
                                if self.status.cooler_boost {
                                    theme.red
                                } else {
                                    theme.muted
                                },
                            )),
                    ),
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
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child(self.t("系统详情", "System details")),
                            )
                            .child(self.badge(format!("EC {}", self.status.fw), theme.accent)),
                    )
                    .child(div().flex().gap_6().child(left).child(right)),
            )
    }

}
