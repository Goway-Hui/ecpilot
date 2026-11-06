//! overview 页面。

use super::*;

impl AppState {
    /// 概览页：4 个实时指标、系统详情、模式与温控摘要。
    pub(super) fn overview(&mut self, cx: &mut Context<Self>) -> Div {
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
        let battery_color = if self.status.battery <= 20 {
            theme.red
        } else {
            theme.green
        };

        let cpu = self.stat_card(
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
            20.0,
            100.0,
        );
        let gpu = self.stat_card(
            self.t("GPU", "GPU"),
            self.status.gpu_util.to_string(),
            "%".into(),
            format!(
                "NV {} °C · EC {} °C · {} MiB",
                self.status.gpu_temp_nv, self.status.gpu_temp, self.status.gpu_mem_used
            ),
            theme.peach,
            &self.gpu_history,
            0.0,
            100.0,
        );
        let fans = self.stat_card(
            self.t("风扇", "Fans"),
            self.status.cpu_fan.to_string(),
            "RPM".into(),
            format!("GPU {} RPM", self.status.gpu_fan),
            theme.green,
            &self.fan_history,
            0.0,
            100.0,
        );
        let battery = self.stat_card(
            self.t("电池", "Battery"),
            self.status.battery.to_string(),
            "%".into(),
            format!(
                "{} · {} {}%",
                self.status.charging,
                self.t("上限", "limit"),
                self.status.threshold
            ),
            battery_color,
            &self.battery_history,
            0.0,
            100.0,
        );
        let kpis = if self.compact {
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(div().flex().gap(px(12.0)).child(cpu).child(gpu))
                .child(div().flex().gap(px(12.0)).child(fans).child(battery))
        } else {
            div()
                .flex()
                .gap(px(12.0))
                .child(cpu)
                .child(gpu)
                .child(fans)
                .child(battery)
        };

        let details = &self.details;
        let left = div()
            .flex()
            .flex_col()
            .child(self.info_line("OS", details.os.clone()))
            .child(self.info_line(
                self.t("内核", "Kernel").as_str(),
                details.kernel.clone(),
            ))
            .child(self.info_line(
                self.t("包数", "Packages").as_str(),
                details.packages.clone(),
            ))
            .child(self.info_line(
                self.t("系统年龄", "Age").as_str(),
                details.age.clone(),
            ))
            .child(self.info_line(
                self.t("用户", "User").as_str(),
                details.user.clone(),
            ))
            .child(self.info_line("PC", details.pc.clone()))
            .child(self.info_line("WM", details.wm.clone()))
            .child(self.info_line("Shell", details.shell.clone()))
            .child(self.info_line(
                self.t("终端", "Terminal").as_str(),
                details.term.clone(),
            ))
            .child(self.info_line(
                self.t("字体", "Font").as_str(),
                details.font.clone(),
            ));

        let mut right = div()
            .flex()
            .flex_col()
            .child(self.info_line("CPU", details.cpu.clone()))
            .child(self.info_line(
                self.t("内存", "Memory").as_str(),
                details.mem.clone(),
            ))
            .child(self.info_line(
                self.t("交换", "Swap").as_str(),
                details.swp.clone(),
            ))
            .child(self.info_line(
                "NVIDIA",
                format!(
                    "{} · {} °C",
                    self.status.gpu_name, self.status.gpu_temp_nv
                ),
            ))
            .child(self.info_line("AMD", details.gpu_amd.clone()))
            .child(self.info_line(
                self.t("风扇", "Fans").as_str(),
                format!("{} / {} RPM", self.status.cpu_fan, self.status.gpu_fan),
            ))
            .child(self.info_line(
                self.t("显示器", "Monitor").as_str(),
                details.mon.clone(),
            ));
        for disk in &details.disks {
            right = right.child(self.info_line(
                self.t("磁盘", "Disk").as_str(),
                disk.clone(),
            ));
        }

        let system = self
            .card()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(self.card_title("系统详情", "System details"))
                    .child(self.hint(self.t("每秒刷新", "updates every 1 s"))),
            )
            .child(
                div()
                    .flex()
                    .min_w(px(0.0))
                    .gap(px(40.0))
                    .child(left.flex_1().min_w(px(0.0)))
                    .child(right.flex_1().min_w(px(0.0))),
            );

        let mode = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(self.card_title("模式与设备", "Mode & devices"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(24.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(theme.text))
                            .child(self.status.fan_mode.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(theme.muted))
                            .child(format!(
                                "{} {} · {} {}",
                                self.t("性能", "Shift"),
                                self.status.shift_mode,
                                self.t("风扇", "Fan"),
                                self.status.cpu_fan
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(self.info_row(
                        self.t("Cooler Boost", "Cooler Boost").as_str(),
                        if self.status.cooler_boost {
                            self.t("开启", "on")
                        } else {
                            self.t("关闭", "off")
                        },
                        if self.status.cooler_boost {
                            theme.red
                        } else {
                            theme.faint
                        },
                    ))
                    .child(self.info_row(
                        "GPU MUX",
                        self.mux.clone(),
                        if self.mux == "Discrete" {
                            theme.green
                        } else {
                            theme.accent
                        },
                    ))
                    .child(self.info_row(
                        self.t("摄像头", "Webcam").as_str(),
                        if self.status.webcam {
                            self.t("开启", "on")
                        } else {
                            self.t("关闭", "off")
                        },
                        if self.status.webcam {
                            theme.green
                        } else {
                            theme.faint
                        },
                    ))
                    .child(self.info_row(
                        "USB Power Share",
                        if self.status.usb_power_share {
                            self.t("开启", "on")
                        } else {
                            self.t("关闭", "off")
                        },
                        if self.status.usb_power_share {
                            theme.green
                        } else {
                            theme.faint
                        },
                    ))
                    .child(self.info_row(
                        "Fn / Win",
                        if self.status.fn_swapped {
                            self.t("已交换", "swapped")
                        } else {
                            self.t("默认", "default")
                        },
                        if self.status.fn_swapped {
                            theme.accent
                        } else {
                            theme.faint
                        },
                    ))
                    .child(self.info_row(
                        self.t("电池上限", "Battery limit").as_str(),
                        format!("{}%", self.status.threshold),
                        theme.faint,
                    ))
                    .child(self.info_row(
                        self.t("充电状态", "Charging").as_str(),
                        self.status.charging.clone(),
                        theme.faint,
                    ))
                    .child(self.info_row(
                        self.t("键盘", "Keyboard").as_str(),
                        self.klc
                            .as_ref()
                            .map(|(_, model)| model.clone())
                            .unwrap_or_else(|| self.t("未检测", "not detected")),
                        theme.faint,
                    )),
            );

        let (level_text, level_color) = match self.status.thermal_level {
            0 => (self.t("正常", "Normal"), theme.green),
            1 => (self.t("降频", "Cautious"), theme.yellow),
            2 => (self.t("深度降频", "Aggressive"), theme.peach),
            _ => (self.t("紧急锁存", "Emergency"), theme.red),
        };
        let cap = if self.status.thermal_cap_mhz > 0 {
            format!("{:.1} GHz", self.status.thermal_cap_mhz as f32 / 1000.0)
        } else {
            self.t("不限制", "unlimited")
        };
        let thermal = self
            .card()
            .flex_1()
            .id("overview-thermal")
            .flex()
            .flex_col()
            .gap(px(10.0))
            .cursor_pointer()
            .hover(move |style| style.border_color(rgb(theme.accent)))
            .on_click(cx.listener(|this, _, _, cx| {
                this.page = Page::Thermal;
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(self.card_title("温控保护", "Thermal guard"))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(div().size(px(6.0)).rounded_full().bg(rgb(level_color)))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(level_color))
                                    .child(level_text),
                            ),
                    ),
            )
            .child(self.level_bar(self.status.thermal_level, level_color))
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(self.temp_metric("Tctl", self.status.tctl, temp_color(self.status.tctl)))
                    .child(self.temp_metric(
                        "Tccd1",
                        self.status.tccd1,
                        temp_color(self.status.tccd1),
                    ))
                    .child(self.temp_metric(
                        "Tccd2",
                        self.status.tccd2,
                        temp_color(self.status.tccd2),
                    )),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(self.hint(format!(
                        "{} {} · boost {}",
                        self.t("上限", "cap"),
                        cap,
                        if self.status.thermal_boost {
                            self.t("开", "on")
                        } else {
                            self.t("关", "off")
                        }
                    )))
                    .child(self.hint(if self.status.thermal_active {
                        self.t("helper 运行中", "helper running")
                    } else {
                        self.t("未启用", "disabled")
                    })),
            )
            .child(div().h(px(1.0)).w_full().bg(rgb(theme.border)))
            .child(self.info_row(
                self.t("写入错误", "Write error").as_str(),
                if self.status.thermal_error.is_empty() {
                    self.t("无", "none")
                } else {
                    self.t("有", "yes")
                },
                if self.status.thermal_error.is_empty() {
                    theme.green
                } else {
                    theme.red
                },
            ))
            .child(self.hint(format!(
                "{} {} / {} / {} °C · {} {} / {} / {} °C",
                self.t("触发", "trigger"),
                self.status.thermal_config.up1,
                self.status.thermal_config.up2,
                self.status.thermal_config.up3,
                self.t("回落", "drop"),
                self.status.thermal_config.down1,
                self.status.thermal_config.down2,
                self.status.thermal_config.down3
            )))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(self.hint(format!(
                        "{} {:.1} GHz · {} {} ms",
                        self.t("安全档", "safe"),
                        self.status.thermal_config.base_khz as f32 / 1_000_000.0,
                        self.t("采样", "sample"),
                        self.status.thermal_config.sample_ms
                    )))
                    .child(self.hint(self.t("查看规则 →", "view rules →"))),
            );

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(kpis)
            .child(system)
            .child(
                div()
                    .flex()
                    .items_stretch()
                    .gap(px(14.0))
                    .child(mode)
                    .child(thermal),
            )
    }

    /// 温控卡内的小温度单元。
    fn temp_metric(&self, label: &str, value: i32, color: u32) -> Div {
        let theme = self.theme();
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .px(px(10.0))
            .py(px(8.0))
            .rounded(px(crate::theme::RADIUS_CONTROL))
            .bg(rgb(theme.bg))
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(theme.faint))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .text_size(px(18.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(color))
                    .child(format!("{value} °C")),
            )
    }
}
