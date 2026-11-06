//! thermal 页面。

use super::*;

impl AppState {
    /// 温控保护页：显示档位、温度，并支持启停。
    pub(super) fn thermal(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let level = self.status.thermal_level;
        let manual = self.status.thermal_manual;
        let active = self.status.thermal_active;
        let level_name = |value: i32| match value {
            0 => self.t("正常", "Normal"),
            1 => self.t("降档 1", "Level 1"),
            2 => self.t("降档 2", "Level 2"),
            3 => self.t("紧急锁存", "Emergency latch"),
            _ => self.t("自定义", "Custom"),
        };
        let (level_text, level_color) = if manual {
            (
                format!("{} · {}", self.t("手动", "Manual"), level_name(level)),
                theme.accent,
            )
        } else {
            (
                level_name(level),
                match level {
                    0 => theme.green,
                    1 => theme.yellow,
                    2 => theme.peach,
                    _ => theme.red,
                },
            )
        };
        let cap = if self.status.thermal_cap_mhz > 0 {
            format!("{:.1} GHz", self.status.thermal_cap_mhz as f32 / 1000.0)
        } else {
            self.t("不限制", "unlimited")
        };
        let temp_color = |value: i32| {
            if value >= 97 {
                theme.red
            } else if value >= 90 {
                theme.yellow
            } else {
                theme.green
            }
        };
        let sensor = |label: &str, value: i32| {
            self.card()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(self.card_title(label, label))
                .child(
                    div()
                        .text_size(px(30.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(temp_color(value)))
                        .child(value.to_string()),
                )
                .child(self.hint("°C".into()))
        };
        let tctl = sensor("Tctl", self.status.tctl);
        let tccd1 = sensor("Tccd1", self.status.tccd1);
        let tccd2 = sensor("Tccd2", self.status.tccd2);

        let level_card = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(self.card_title("当前档位", "Current level"))
            .child(
                div()
                    .text_size(px(22.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(level_color))
                    .child(level_text.clone()),
            )
            .child(self.level_bar(level, level_color))
            .child(self.hint(format!(
                "{} {} · boost {}",
                self.t("上限", "cap"),
                cap,
                if self.status.thermal_boost {
                    self.t("开", "on")
                } else {
                    self.t("关", "off")
                }
            )));

        let rules = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(self.card_title("档位规则", "Level rules"))
            .child({
                let config = &self.status.thermal_config;
                let cap = |khz: i32| khz as f32 / 1_000_000.0;
                self.hint(if self.lang == Lang::Zh {
                    format!(
                        "0：≤{}°C → {:.1} GHz，boost 开\n1：≥{}°C → {:.1} GHz\n2：≥{}°C → {:.1} GHz，boost 关\n3：≥{}°C → {:.1} GHz，热锁存",
                        config.up1, cap(config.cap0), config.up1, cap(config.cap1),
                        config.up2, cap(config.cap2), config.up3, cap(config.cap3)
                    )
                } else {
                    format!(
                        "0: <= {}°C -> {:.1} GHz, boost on\n1: >= {}°C -> {:.1} GHz\n2: >= {}°C -> {:.1} GHz, boost off\n3: >= {}°C -> {:.1} GHz, thermal latch",
                        config.up1, cap(config.cap0), config.up1, cap(config.cap1),
                        config.up2, cap(config.cap2), config.up3, cap(config.cap3)
                    )
                })
            })
            .child({
                let config = &self.status.thermal_config;
                self.hint(if self.lang == Lang::Zh {
                    format!(
                        "回落 ≤{}/{}/{}°C · 停留 {} ms\n锁存 {}°C/{}s · 采样 {} ms · 安全档 {:.1} GHz",
                        config.down1, config.down2, config.down3,
                        config.dwell_ms, config.latch_temp, config.latch_secs, config.sample_ms,
                        config.base_khz as f32 / 1_000_000.0
                    )
                } else {
                    format!(
                        "down <= {}/{}/{}°C · dwell {} ms\nlatch {}°C/{}s · sample {} ms · safe {:.1} GHz",
                        config.down1, config.down2, config.down3,
                        config.dwell_ms, config.latch_temp, config.latch_secs, config.sample_ms,
                        config.base_khz as f32 / 1_000_000.0
                    )
                })
            });

        let toggle: AnyElement = if active {
            self.action_button(
                "thermal-toggle",
                self.t("停用保护", "Disable"),
                true,
                cx,
                |this, cx| {
                    let on = !this.status.thermal_active;
                    this.run_action(cx, move || ec::set_thermal_guard(on))
                },
            )
            .into_any_element()
        } else {
            self.primary_button(
                "thermal-toggle",
                self.t("启用保护", "Enable"),
                cx,
                |this, cx| {
                    let on = !this.status.thermal_active;
                    this.run_action(cx, move || ec::set_thermal_guard(on))
                },
            )
            .into_any_element()
        };

        let control = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(self.card_title("保护控制", "Guard control"))
            .child(self.info_row(
                self.t("服务", "Service").as_str(),
                if active {
                    self.t("运行中", "running")
                } else {
                    self.t("已停止", "stopped")
                },
                if active { theme.green } else { theme.red },
            ))
            .child(self.info_row(
                self.t("档位", "Level").as_str(),
                level_text.clone(),
                level_color,
            ))
            .child(self.info_row(
                self.t("频率上限", "Frequency cap").as_str(),
                cap.clone(),
                theme.accent,
            ))
            .child(self.info_row(
                "Boost",
                if self.status.thermal_boost {
                    self.t("开", "on")
                } else {
                    self.t("关", "off")
                },
                if self.status.thermal_boost {
                    theme.green
                } else {
                    theme.faint
                },
            ))
            .child(
                div().flex().gap(px(8.0)).pt(px(4.0)).child(toggle).child(
                    self.action_button(
                        "thermal-restart",
                        self.t("重启保护", "Restart"),
                        false,
                        cx,
                        |this, cx| this.run_action(cx, ec::restart_thermal_guard),
                    ),
                ),
            )
            .child(self.hint(self.t(
                "helper 内置线程运行；关闭应用不会停止保护。",
                "Runs in the helper; closing the app does not stop protection.",
            )));

        let manual_card = self.manual_card(cx);

        let sensors = if self.compact {
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(div().flex().gap(px(12.0)).child(tctl).child(tccd1))
                .child(div().flex().gap(px(12.0)).child(tccd2).child(level_card))
        } else {
            div()
                .flex()
                .gap(px(14.0))
                .child(tctl)
                .child(tccd1)
                .child(tccd2)
                .child(level_card)
        };
        let mut page = div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(sensors)
            .child(manual_card)
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .items_stretch()
                    .child(rules)
                    .child(control),
            );

        if !self.status.thermal_error.is_empty() {
            page = page.child(
                self.card()
                    .bg(rgb(theme.error_bg))
                    .border_color(rgb(theme.red))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(rgb(theme.error_text))
                            .child(format!(
                                "{} {}",
                                self.t("温控写入错误：", "Thermal write error: "),
                                self.status.thermal_error
                            )),
                    ),
            );
        }
        if !active {
            page = page.child(
                self.card()
                    .bg(rgb(theme.error_bg))
                    .border_color(rgb(theme.red))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(rgb(theme.error_text))
                            .child(self.t(
                                "当前未启用温度保护；长时间满载可能触发热断电。",
                                "Thermal protection is off; sustained load may trigger a thermal shutdown.",
                            )),
                    ),
            );
        }
        page
    }

    /// 手动温控卡：开关 + 档位、频率和 boost 控制。
    fn manual_card(&mut self, cx: &mut Context<Self>) -> Div {
        let manual = self.status.thermal_manual;
        let body: AnyElement = if manual {
            self.manual_body(cx).into_any_element()
        } else {
            self.hint(self.t(
                "自动模式：档位由温度和停留时间决定。",
                "Automatic mode: level follows temperature and dwell time.",
            ))
            .into_any_element()
        };
        self.card()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.card_title("手动控制", "Manual control"))
                            .child(self.hint(if manual {
                                self.t(
                                    "手动模式绕过自动降温；长时间使用请留意温度。",
                                    "Manual mode bypasses automatic downshifts; watch the temperature.",
                                )
                            } else {
                                self.t(
                                    "打开后可固定档位与频率上限。",
                                    "Turn it on to pin a level and a frequency cap.",
                                )
                            })),
                    )
                    .child(self.switch("thermal-manual", manual, cx, |this, cx| {
                        let on = !this.status.thermal_manual;
                        this.run_action(cx, move || ec::set_thermal_manual(on));
                    })),
            )
            .child(body)
    }

    fn manual_body(&mut self, cx: &mut Context<Self>) -> Div {
        let level = self.status.thermal_level;
        let cap_mhz = self.status.thermal_cap_mhz;
        let boost = self.status.thermal_boost;
        let base_mhz = self.status.thermal_config.base_khz / 1000;

        let levels = [
            ("thermal-level-0", 0, "正常", "Normal"),
            ("thermal-level-1", 1, "降档 1", "Level 1"),
            ("thermal-level-2", 2, "降档 2", "Level 2"),
            ("thermal-level-3", 3, "紧急锁存", "Emergency"),
        ];
        let mut level_row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.0))
            .child(self.hint(self.t("档位", "Level")));
        for (id, value, zh, en) in levels {
            level_row = level_row.child(self.pill(
                id,
                self.t(zh, en),
                self.status.thermal_manual && level == value,
                cx,
                move |this, cx| this.run_action(cx, move || ec::set_thermal_level(value)),
            ));
        }

        let presets = [
            ("thermal-freq-1500", 1500),
            ("thermal-freq-2400", 2400),
            ("thermal-freq-3800", 3800),
            ("thermal-freq-4200", 4200),
        ];
        let mut preset_row = div()
            .flex()
            .flex_wrap()
            .gap(px(6.0))
            .child(self.hint(self.t("预设", "Presets")));
        for (id, mhz) in presets {
            preset_row = preset_row.child(self.pill(
                id,
                format!("{:.1} GHz", mhz as f32 / 1000.0),
                cap_mhz == mhz,
                cx,
                move |this, cx| {
                    let boost = this.status.thermal_boost || mhz > base_mhz;
                    this.run_action(cx, move || {
                        ec::set_thermal_frequency(mhz * 1000, boost)
                    });
                },
            ));
        }

        let down = (cap_mhz - 100).max(800);
        let up = (cap_mhz + 100).min(5000);
        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(level_row)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(8.0))
                    .child(self.hint(self.t("频率上限", "Frequency cap")))
                    .child(
                        div()
                            .text_size(px(20.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(self.theme().text))
                            .child(format!("{:.1} GHz", cap_mhz as f32 / 1000.0)),
                    )
                    .child(self.action_button(
                        "thermal-freq-down",
                        "− 100 MHz".into(),
                        false,
                        cx,
                        move |this, cx| {
                            let boost = this.status.thermal_boost;
                            this.run_action(cx, move || {
                                ec::set_thermal_frequency(down * 1000, boost)
                            });
                        },
                    ))
                    .child(self.action_button(
                        "thermal-freq-up",
                        "+ 100 MHz".into(),
                        false,
                        cx,
                        move |this, cx| {
                            let boost = this.status.thermal_boost || up > base_mhz;
                            this.run_action(cx, move || {
                                ec::set_thermal_frequency(up * 1000, boost)
                            });
                        },
                    )),
            )
            .child(preset_row)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(self.hint("Boost".into()))
                    .child(self.switch("thermal-boost", boost, cx, |this, cx| {
                        let cap = this.status.thermal_cap_mhz.max(800);
                        let next = !this.status.thermal_boost;
                        this.run_action(cx, move || {
                            ec::set_thermal_frequency(cap * 1000, next)
                        });
                    }))
                    .child(self.hint(if boost {
                        self.t("允许睿频", "boost allowed")
                    } else {
                        self.t("固定上限，不允许 boost", "capped, boost off")
                    })),
            )
            .child(self.hint(format!(
                "{} {:.1} GHz；{}",
                self.t(
                    "关闭 boost 时上限为基础频率",
                    "With boost off the cap is the base frequency"
                ),
                base_mhz as f32 / 1000.0,
                self.t(
                    "提高上限会自动开启 boost，超出硬件范围时由 helper 截断。",
                    "raising the cap turns boost on; the helper clamps it to the CPU range."
                )
            )))
    }
}
