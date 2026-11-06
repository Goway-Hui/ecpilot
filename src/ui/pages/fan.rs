//! fan 页面。

use super::*;

impl AppState {
    /// 风扇控制页。
    pub(super) fn fan(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = self.theme();
        let presets = [
            ("default", "默认", "Default", "均衡", "Balanced"),
            ("quiet", "静音", "Quiet", "低噪", "Quiet"),
            ("gaming", "游戏", "Gaming", "激进", "Aggressive"),
            ("max", "最大", "Max", "全速", "Full speed"),
        ];
        let mode = self.status.fan_mode.clone();
        let mut presets_grid = div().flex().flex_col().gap_3();
        for pair in presets.chunks(2) {
            let mut row = div().flex().gap_3();
            for &(id, zh, en, desc_zh, desc_en) in pair {
                let label = self.t(zh, en);
                let desc = self.t(desc_zh, desc_en);
                let active = self.fan_preset.as_deref() == Some(id);
                let mut card = self
                    .card()
                    .id(id)
                    .flex_1()
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(self.theme().card_hover)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.run_action_then(
                            cx,
                            move || {
                                let result = ec::apply_preset(id);
                                if result.is_ok() {
                                    save_fan_preset(id);
                                    save_fan_mode("advanced");
                                }
                                result
                            },
                            move |this| this.fan_preset = Some(id.to_string()),
                        );
                    }))
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(label),
                    )
                    .child(self.label(desc));
                if active {
                    card = card
                        .border_color(rgb(self.theme().accent))
                        .child(self.label(self.t("已应用", "Active")));
                }
                row = row.child(card);
            }
            presets_grid = presets_grid.child(row);
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("风扇模式", "Fan mode")))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.pill(
                                "fan-auto",
                                "Auto".into(),
                                mode == "auto",
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || {
                                        let result = ec::set_fan_mode("auto");
                                        if result.is_ok() {
                                            save_fan_mode("auto");
                                        }
                                        result
                                    });
                                },
                            ))
                            .child(self.pill(
                                "fan-silent",
                                "Silent".into(),
                                mode == "silent",
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || {
                                        let result = ec::set_fan_mode("silent");
                                        if result.is_ok() {
                                            save_fan_mode("silent");
                                        }
                                        result
                                    });
                                },
                            ))
                            .child(self.pill(
                                "fan-advanced",
                                "Advanced".into(),
                                mode == "advanced",
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || {
                                        let result = ec::set_fan_mode("advanced");
                                        if result.is_ok() {
                                            save_fan_mode("advanced");
                                        }
                                        result
                                    });
                                },
                            )),
                    ),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("性能模式", "Shift mode")))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.pill(
                                "shift-eco",
                                "Eco".into(),
                                self.status.shift_mode == "eco",
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || {
                                        let result = ec::set_shift_mode("eco");
                                        if result.is_ok() {
                                            save_shift_mode("eco");
                                        }
                                        result
                                    });
                                },
                            ))
                            .child(self.pill(
                                "shift-comfort",
                                "Comfort".into(),
                                self.status.shift_mode == "comfort",
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || {
                                        let result = ec::set_shift_mode("comfort");
                                        if result.is_ok() {
                                            save_shift_mode("comfort");
                                        }
                                        result
                                    });
                                },
                            ))
                            .child(self.pill(
                                "shift-turbo",
                                "Turbo".into(),
                                self.status.shift_mode == "turbo",
                                cx,
                                |this, cx| {
                                    this.run_action(cx, || {
                                        let result = ec::set_shift_mode("turbo");
                                        if result.is_ok() {
                                            save_shift_mode("turbo");
                                        }
                                        result
                                    });
                                },
                            )),
                    ),
            )
            .child(
                self.card()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.label(self.t("曲线预设", "Curve presets")))
                    .child(presets_grid),
            )
            .child(
                self.card()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(self.label(self.t("Cooler Boost", "Cooler Boost")))
                            .child(self.badge(
                                if self.status.cooler_boost {
                                    self.t("开启", "on")
                                } else {
                                    self.t("关闭", "off")
                                },
                                if self.status.cooler_boost { self.theme().red } else { self.theme().muted },
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.action_button(
                                "cooler-on",
                                self.t("开启", "Turn on"),
                                false,
                                cx,
                                |this, cx| this.run_action(cx, || ec::set_cooler_boost(true)),
                            ))
                            .child(self.action_button(
                                "cooler-off",
                                self.t("关闭", "Turn off"),
                                true,
                                cx,
                                |this, cx| this.run_action(cx, || ec::set_cooler_boost(false)),
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
                                    .child(self.t("自定义曲线", "Custom curve")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(self.pill(
                                        "curve-cpu",
                                        "CPU".into(),
                                        self.curve_target == CurveTarget::Cpu,
                                        cx,
                                        |this, cx| {
                                            this.curve_target = CurveTarget::Cpu;
                                            cx.notify();
                                        },
                                    ))
                                    .child(self.pill(
                                        "curve-gpu",
                                        "GPU".into(),
                                        self.curve_target == CurveTarget::Gpu,
                                        cx,
                                        |this, cx| {
                                            this.curve_target = CurveTarget::Gpu;
                                            cx.notify();
                                        },
                                    )),
                            ),
                    )
                    .child({
                        let (temps, speeds) = match self.curve_target {
                            CurveTarget::Cpu => (&self.custom.cpu_temp, &self.custom.cpu_speed),
                            CurveTarget::Gpu => (&self.custom.gpu_temp, &self.custom.gpu_speed),
                        };
                        self.curve_chart(temps, speeds)
                    })
                    .child({
                        let (temps, _) = match self.curve_target {
                            CurveTarget::Cpu => (&self.custom.cpu_temp, &self.custom.cpu_speed),
                            CurveTarget::Gpu => (&self.custom.gpu_temp, &self.custom.gpu_speed),
                        };
                        let mut row = div().flex().items_center().gap_2();
                        row = row.child(
                            div()
                                .w(px(44.0))
                                .text_size(px(12.0))
                                .text_color(rgb(theme.muted))
                                .child(self.t("温度", "Temp")),
                        );
                        for (index, temp) in temps.iter().enumerate().take(6) {
                            row = row.child(self.stepper(
                                "curve-temp",
                                index,
                                *temp,
                                true,
                                cx,
                            ));
                        }
                        row
                    })
                    .child({
                        let (_, speeds) = match self.curve_target {
                            CurveTarget::Cpu => (&self.custom.cpu_temp, &self.custom.cpu_speed),
                            CurveTarget::Gpu => (&self.custom.gpu_temp, &self.custom.gpu_speed),
                        };
                        let mut row = div().flex().items_center().gap_2();
                        row = row.child(
                            div()
                                .w(px(44.0))
                                .text_size(px(12.0))
                                .text_color(rgb(theme.muted))
                                .child(self.t("转速", "Speed")),
                        );
                        for (index, speed) in speeds.iter().enumerate().take(7) {
                            row = row.child(self.stepper(
                                "curve-speed",
                                index,
                                *speed,
                                false,
                                cx,
                            ));
                        }
                        row
                    })
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(self.action_button(
                                "reset-custom",
                                self.t("重置为默认", "Reset to default"),
                                false,
                                cx,
                                |this, cx| {
                                    this.custom = CustomCurve::default();
                                    save_custom(&this.custom);
                                    cx.notify();
                                },
                            ))
                            .child(self.action_button(
                                "apply-custom",
                                self.t("应用自定义曲线", "Apply custom curve"),
                                false,
                                cx,
                                |this, cx| {
                                    let custom = this.custom.clone();
                                    this.run_action_then(
                                        cx,
                                        move || {
                                            let result = ec::apply_custom(
                                                &custom.cpu_temp,
                                                &custom.cpu_speed,
                                                &custom.gpu_temp,
                                                &custom.gpu_speed,
                                            );
                                            if result.is_ok() {
                                                save_fan_preset("custom");
                                                save_fan_mode("advanced");
                                            }
                                            save_custom(&custom);
                                            result
                                        },
                                        |this| this.fan_preset = Some("custom".into()),
                                    );
                                },
                            )),
                    ),
            )
    }

}
