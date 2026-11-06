//! thermal 页面。

use super::*;

impl AppState {
    /// 温控保护页：显示档位、温度，并支持启停。
    pub(super) fn thermal(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let level = self.status.thermal_level;
        let active = self.status.thermal_active;
        let level_name = |value: i32| match value {
            0 => self.t("正常", "Normal"),
            1 => self.t("降档 1", "Level 1"),
            2 => self.t("降档 2", "Level 2"),
            _ => self.t("紧急锁存", "Emergency latch"),
        };
        let (level_text, level_color) = (
            level_name(level),
            match level {
                0 => theme.green,
                1 => theme.yellow,
                2 => theme.peach,
                _ => theme.red,
            },
        );
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

        let rules_card = self.rules_card(cx);

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
            .child(rules_card)
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

    /// 档位预设编辑卡：调整各档触发温度与频率上限，保存后立即生效并持久化。
    fn rules_card(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let base_mhz = self.status.thermal_config.base_khz / 1000;

        let cfg = self.rules_edit.unwrap_or(self.status.thermal_config);
        let triggers = [
            ("thermal-trig1-down", "thermal-trig1-up", 0usize, self.t("降档 1 触发", "Level 1 at"), cfg.up1),
            ("thermal-trig2-down", "thermal-trig2-up", 1, self.t("降档 2 触发", "Level 2 at"), cfg.up2),
            ("thermal-trig3-down", "thermal-trig3-up", 2, self.t("紧急触发", "Emergency at"), cfg.up3),
        ];
        let mut trigger_col = div()
            .flex_1()
            .min_w(px(250.0))
            .flex()
            .flex_col()
            .gap(px(9.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(self.hint(self.t("触发温度", "Trigger temperatures")))
                    .child(self.hint(self.t(
                        "回落 = 触发 − 5 °C",
                        "down = trigger − 5 °C",
                    ))),
            );
        for (id_down, id_up, index, label, temp) in triggers {
            let step_down = move |this: &mut AppState, cx: &mut Context<AppState>| {
                let draft = this.rules_edit.unwrap_or(this.status.thermal_config);
                let draft = step_trigger(draft, index, -1);
                save_rules(this, cx, draft);
            };
            let step_up = move |this: &mut AppState, cx: &mut Context<AppState>| {
                let draft = this.rules_edit.unwrap_or(this.status.thermal_config);
                let draft = step_trigger(draft, index, 1);
                save_rules(this, cx, draft);
            };
            trigger_col = trigger_col.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .min_w(px(88.0))
                            .text_size(px(12.5))
                            .text_color(rgb(theme.faint))
                            .child(label),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .min_w(px(56.0))
                                    .text_size(px(15.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(theme.text))
                                    .child(format!("{temp} °C")),
                            )
                            .child(
                                self.action_button(id_down, "− 1 °C".into(), false, cx, step_down),
                            )
                            .child(
                                self.action_button(id_up, "+ 1 °C".into(), false, cx, step_up),
                            ),
                    ),
            );
        }

        let caps = [
            ("thermal-cap0-down", "thermal-cap0-up", 0usize, self.t("正常", "Normal"), cfg_cap(self, 0)),
            ("thermal-cap1-down", "thermal-cap1-up", 1, self.t("降档 1", "Level 1"), cfg_cap(self, 1)),
            ("thermal-cap2-down", "thermal-cap2-up", 2, self.t("降档 2", "Level 2"), cfg_cap(self, 2)),
            ("thermal-cap3-down", "thermal-cap3-up", 3, self.t("紧急锁存", "Emergency"), cfg_cap(self, 3)),
        ];
        let mut cap_col = div()
            .flex_1()
            .min_w(px(310.0))
            .flex()
            .flex_col()
            .gap(px(9.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(self.hint(self.t("频率上限", "Frequency caps")))
                    .child(self.hint(format!(
                        "{} / {} {} · {} {:.1} GHz",
                        self.t("正常", "Normal"),
                        self.t("降档 1", "L1"),
                        self.t("boost", "boost"),
                        self.t("基础", "base"),
                        base_mhz as f32 / 1000.0
                    ))),
            );
        for (id_down, id_up, index, label, cap_khz) in caps {
            let cap_mhz = cap_khz / 1000;
            let step_down = move |this: &mut AppState, cx: &mut Context<AppState>| {
                let base = this.status.thermal_config.base_khz / 1000;
                let draft = this.rules_edit.unwrap_or(this.status.thermal_config);
                let draft = step_cap(draft, index, -100, base);
                save_rules(this, cx, draft);
            };
            let step_up = move |this: &mut AppState, cx: &mut Context<AppState>| {
                let base = this.status.thermal_config.base_khz / 1000;
                let draft = this.rules_edit.unwrap_or(this.status.thermal_config);
                let draft = step_cap(draft, index, 100, base);
                save_rules(this, cx, draft);
            };
            cap_col = cap_col.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .min_w(px(88.0))
                            .text_size(px(12.5))
                            .text_color(rgb(theme.faint))
                            .child(label),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .min_w(px(58.0))
                                    .text_size(px(15.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(theme.text))
                                    .child(format!("{:.1} GHz", cap_mhz as f32 / 1000.0)),
                            )
                            .child(
                                self.action_button(
                                    id_down,
                                    "− 100 MHz".into(),
                                    false,
                                    cx,
                                    step_down,
                                ),
                            )
                            .child(
                                self.action_button(
                                    id_up,
                                    "+ 100 MHz".into(),
                                    false,
                                    cx,
                                    step_up,
                                ),
                            ),
                    ),
            );
        }

        let reset = self.action_button(
            "thermal-rules-reset",
            self.t("恢复默认", "Defaults"),
            false,
            cx,
            |this, cx| {
                this.rules_edit = None;
                this.run_action(cx, || ec::set_thermal_rules("{}".into()));
            },
        );

        self.card()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_start()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(self.card_title("预设调整", "Presets"))
                            .child(self.hint(self.t(
                                "各档触发温度与频率上限；保存后立即生效并持久化。",
                                "Per-level triggers and caps; applied and persisted on save.",
                            ))),
                    )
                    .child(reset),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(24.0))
                    .child(trigger_col)
                    .child(cap_col),
            )
    }
}

/// 读取草稿或生效参数中的档位频率（kHz）。
fn cfg_cap(state: &AppState, index: usize) -> i32 {
    let cfg = state.rules_edit.unwrap_or(state.status.thermal_config);
    match index {
        0 => cfg.cap0,
        1 => cfg.cap1,
        2 => cfg.cap2,
        _ => cfg.cap3,
    }
}

/// 保存规则草稿并立即应用到 helper。
fn save_rules(this: &mut AppState, cx: &mut Context<AppState>, draft: ThermalConfig) {
    this.rules_edit = Some(draft);
    let json = serde_json::to_string(&draft).unwrap_or_default();
    this.run_action(cx, move || ec::set_thermal_rules(json));
}

/// 步进某档触发温度并保持升序；回落温度由 helper 派生。
fn step_trigger(mut cfg: ThermalConfig, index: usize, delta: i32) -> ThermalConfig {
    let value = match index {
        0 => cfg.up1 + delta,
        1 => cfg.up2 + delta,
        _ => cfg.up3 + delta,
    };
    match index {
        0 => cfg.up1 = value.clamp(60, cfg.up2 - 1),
        1 => cfg.up2 = value.clamp(cfg.up1 + 1, cfg.up3 - 1),
        _ => cfg.up3 = value.clamp(cfg.up2 + 1, 110),
    }
    cfg
}

/// 步进某档频率上限（MHz），保持非递增并满足 0/1 档 boost 下限。
fn step_cap(mut cfg: ThermalConfig, index: usize, delta_mhz: i32, base_mhz: i32) -> ThermalConfig {
    let to_mhz = |khz: i32| khz / 1000;
    let value = (to_mhz(match index {
        0 => cfg.cap0,
        1 => cfg.cap1,
        2 => cfg.cap2,
        _ => cfg.cap3,
    }) + delta_mhz)
        .clamp(800, 5000);
    match index {
        0 => {
            let lo = to_mhz(cfg.cap1).max(base_mhz);
            cfg.cap0 = value.max(lo) * 1000;
        }
        1 => {
            let lo = to_mhz(cfg.cap2).max(base_mhz);
            let hi = to_mhz(cfg.cap0).min(5000);
            cfg.cap1 = value.max(lo).min(hi) * 1000;
        }
        2 => {
            let lo = to_mhz(cfg.cap3).max(800);
            let hi = to_mhz(cfg.cap1).min(base_mhz);
            cfg.cap2 = value.max(lo).min(hi) * 1000;
        }
        _ => {
            let hi = to_mhz(cfg.cap2).min(base_mhz);
            cfg.cap3 = value.max(800).min(hi) * 1000;
        }
    }
    cfg
}
