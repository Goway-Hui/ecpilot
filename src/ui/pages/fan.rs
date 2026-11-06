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

        let mode_card = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(self.card_title("风扇模式", "Fan mode"))
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
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
            );

        let shift_card = self
            .card()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(self.card_title("性能模式", "Shift mode"))
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
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
            );

        let mut preset_row = div().flex().gap(px(12.0));
        for (id, zh, en, desc_zh, desc_en) in presets {
            let label = self.t(zh, en);
            let desc = self.t(desc_zh, desc_en);
            let active = self.fan_preset.as_deref() == Some(id);
            let mut card = self
                .card()
                .id(id)
                .flex_1()
                .p(px(16.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .cursor_pointer()
                .hover(move |style| style.border_color(rgb(theme.accent)))
                .active(|style| style.opacity(0.9))
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
                        .text_size(px(15.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(theme.text))
                        .child(label),
                )
                .child(self.hint(desc));
            if active {
                card = card
                    .border_color(rgb(theme.accent))
                    .bg(rgb(theme.accent_soft));
            }
            preset_row = preset_row.child(card);
        }

        let cooler_action: AnyElement = if self.status.cooler_boost {
            self.action_button(
                "cooler-toggle",
                self.t("关闭 Cooler Boost", "Turn off Cooler Boost"),
                true,
                cx,
                |this, cx| this.run_action(cx, || ec::set_cooler_boost(false)),
            )
            .into_any_element()
        } else {
            self.primary_button(
                "cooler-toggle",
                self.t("开启 Cooler Boost", "Turn on Cooler Boost"),
                cx,
                |this, cx| this.run_action(cx, || ec::set_cooler_boost(true)),
            )
            .into_any_element()
        };
        let cooler = self
            .card()
            .flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(self.card_title("Cooler Boost", "Cooler Boost"))
                    .child(self.hint(if self.status.cooler_boost {
                        self.t("全速散热已开启", "full-speed cooling on")
                    } else {
                        self.t("按需开启，噪声与温度同时上升", "raises noise and cooling")
                    })),
            )
            .child(cooler_action);

        let custom = self
            .card()
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
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(self.t("自定义曲线", "Custom curve")),
                            )
                            .child(self.hint(self.t(
                                "6 个温度点，7 档转速；最后一段至少 70%",
                                "6 temperature points, 7 speed steps; last at least 70%",
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(6.0))
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
                let mut row = div().flex().items_center().gap(px(4.0));
                row = row.child(
                    div()
                        .w(px(40.0))
                        .text_size(px(12.0))
                        .text_color(rgb(theme.faint))
                        .child(self.t("温度", "Temp")),
                );
                for (index, temp) in temps.iter().enumerate().take(6) {
                    row = row.child(self.stepper("curve-temp", index, *temp, true, cx));
                }
                row
            })
            .child({
                let (_, speeds) = match self.curve_target {
                    CurveTarget::Cpu => (&self.custom.cpu_temp, &self.custom.cpu_speed),
                    CurveTarget::Gpu => (&self.custom.gpu_temp, &self.custom.gpu_speed),
                };
                let mut row = div().flex().items_center().gap(px(4.0));
                row = row.child(
                    div()
                        .w(px(40.0))
                        .text_size(px(12.0))
                        .text_color(rgb(theme.faint))
                        .child(self.t("转速", "Speed")),
                );
                for (index, speed) in speeds.iter().enumerate().take(7) {
                    row = row.child(self.stepper("curve-speed", index, *speed, false, cx));
                }
                row
            })
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.0))
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
                    .child(self.primary_button(
                        "apply-custom",
                        self.t("应用自定义曲线", "Apply custom curve"),
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
            );

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .child(mode_card)
                    .child(shift_card),
            )
            .child(preset_row)
            .child(cooler)
            .child(custom)
    }
}
