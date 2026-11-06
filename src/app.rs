//! 应用状态、定时刷新与页面枚举。
//!
//! UI 渲染在 `ui` 模块中，本模块只负责状态和生命周期。

use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gpui::{
    div, prelude::*, px, rgb, AnyWindowHandle, App, Bounds, Context, IntoElement, Render, Window,
    WindowBounds, WindowOptions, size,
};

use crate::config::{load_close_to_tray, load_config, load_custom, load_fan_settings, CurveTarget, CustomCurve, Lang};
use crate::{details, ec, klc};
use crate::theme::{Theme, ThemeMode, DARK, LIGHT};
use crate::tray::TrayCommand;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Page {
    Overview,
    Fan,
    Keyboard,
    Gpu,
    Telemetry,
    Thermal,
    Settings,
}

/// 应用全局状态：页面、语言、主题、硬件状态与历史曲线。
/// 最近一次使用的页面；托盘重建窗口时恢复。
static LAST_PAGE: Mutex<Option<Page>> = Mutex::new(None);

/// 记录当前页面，供托盘重建窗口使用。
pub(crate) fn remember_page(page: Page) {
    if let Ok(mut guard) = LAST_PAGE.lock() {
        *guard = Some(page);
    }
}

fn last_page() -> Page {
    match LAST_PAGE.lock() {
        Ok(guard) => guard.unwrap_or(Page::Overview),
        Err(_) => Page::Overview,
    }
}

/// 主窗口句柄共享槽；窗口关闭后变为无效，托盘 Show 会尝试重建。
pub(crate) type TrayWindows = Arc<Mutex<Option<AnyWindowHandle>>>;

/// 创建窗口句柄共享槽。
pub(crate) fn new_tray_windows() -> TrayWindows {
    Arc::new(Mutex::new(None))
}

/// 启动托盘命令轮询：Show 恢复/重建窗口，Quit 退出应用。
static IPC_SHOW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

extern "C" fn handle_ipc_signal(_: i32) {
    IPC_SHOW.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn install_ipc_signals() {
    use nix::sys::signal::{signal, SigHandler, Signal};
    // SAFETY: handler only sets an atomic flag.
    unsafe {
        let _ = signal(Signal::SIGUSR1, SigHandler::Handler(handle_ipc_signal));
    }
}

pub(crate) fn spawn_tray_polling(
    cx: &mut App,
    rx: std::sync::mpsc::Receiver<TrayCommand>,
    windows: TrayWindows,
) {
    install_ipc_signals();
    cx.spawn(async move |cx| loop {
        if IPC_SHOW.swap(false, std::sync::atomic::Ordering::Relaxed) {
            cx.update(|cx| show_or_open(cx, &windows));
        }
        while let Ok(command) = rx.try_recv() {
            match command {
                TrayCommand::Quit => std::process::exit(0),
                TrayCommand::Show => {
                    cx.update(|cx| show_or_open(cx, &windows));
                }
            }
        }
        cx.background_executor().timer(Duration::from_millis(200)).await;
    })
    .detach();
}

/// 已有窗口则激活；窗口不存在（已关闭）则重建。
fn show_or_open(cx: &mut App, windows: &TrayWindows) {
    if let Some(handle) = windows.lock().ok().and_then(|guard| *guard) {
        if cx
            .update_window(handle, |_, window, cx| {
                window.activate_window();
                cx.activate(true);
            })
            .is_ok()
        {
            return;
        }
    }
    let page = last_page();
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1180.0), px(980.0)),
            cx,
        ))),
        ..Default::default()
    };
    let closure_windows = Arc::clone(windows);
    if let Ok(handle) = cx.open_window(options, move |window, cx| {
        *closure_windows.lock().unwrap() = Some(window.window_handle());
        cx.new(move |cx| AppState::new(cx, page))
    }) {
        *windows.lock().unwrap() = Some(handle.into());
    }
}

/// 后台 I/O 任务：特权操作或外部命令，在独立线程串行执行。
pub(crate) type Job = Box<dyn FnOnce() + Send + 'static>;

/// 后台线程读到的硬件快照；UI 只负责应用，不做阻塞 I/O。
struct Snapshot {
    status: ec::Status,
    mux: String,
    mem: String,
    swp: String,
    disks: Vec<String>,
}

fn read_snapshot() -> Snapshot {
    let dynamic = details::read_dynamic();
    Snapshot {
        status: ec::read_status(),
        mux: ec::mux_mode(),
        mem: dynamic.mem,
        swp: dynamic.swp,
        disks: dynamic.disks,
    }
}

/// 串行执行特权任务，并每秒采样一次硬件状态。
fn io_worker(jobs: Receiver<Job>, events: Sender<Snapshot>) {
    if events.send(read_snapshot()).is_err() {
        return;
    }
    loop {
        match jobs.recv_timeout(Duration::from_secs(1)) {
            Ok(job) => {
                let _ = catch_unwind(AssertUnwindSafe(job));
                if events.send(read_snapshot()).is_err() {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if events.send(read_snapshot()).is_err() {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

pub(crate) struct AppState {
    /// 当前页面。
    pub(crate) page: Page,
    /// 界面语言。
    pub(crate) lang: Lang,
    /// 主题模式。
    pub(crate) theme_mode: ThemeMode,
    /// 自定义风扇曲线。
    pub(crate) custom: CustomCurve,
    /// 曲线编辑目标。
    pub(crate) curve_target: CurveTarget,
    /// EC/硬件实时状态。
    pub(crate) status: ec::Status,
    /// GPU MUX 状态。
    pub(crate) mux: String,
    /// 检测到的 KLC 键盘。
    pub(crate) klc: Option<(String, String)>,
    /// 键盘亮度。
    pub(crate) brightness: u8,
    /// 键盘颜色。
    pub(crate) color: (u8, u8, u8),
    /// 操作结果提示。
    pub(crate) message: String,
    /// 提示是否为成功。
    pub(crate) message_ok: bool,
    /// 系统详情。
    pub(crate) details: details::Details,
    /// CPU 使用率历史。
    pub(crate) cpu_history: VecDeque<f32>,
    /// GPU 使用率历史。
    pub(crate) gpu_history: VecDeque<f32>,
    /// 风扇转速历史。
    pub(crate) fan_history: VecDeque<f32>,
    /// 电池电量历史。
    pub(crate) battery_history: VecDeque<f32>,
    /// GPUI 焦点句柄。
    pub(crate) focus: gpui::FocusHandle,
    /// 窗口是否获得焦点。
    pub(crate) focused: bool,
    /// 关闭窗口时是否最小化到托盘。
    pub(crate) close_to_tray: bool,
    /// 上次应用的曲线预设，用于高亮当前预设。
    pub(crate) fan_preset: Option<String>,
    /// 上次看到的温控档位；用于换档通知。
    thermal_level_seen: Option<i32>,
    /// 上次高温提醒时间（15 分钟限流）。
    last_hot_notify: Option<Instant>,
    /// 后台 I/O 线程的任务队列。
    jobs: Sender<Job>,
}


impl AppState {
    /// 创建状态，并启动后台采样/特权操作线程。
    pub(crate) fn new(cx: &mut Context<Self>, page: Page) -> Self {
        crate::config::migrate_legacy_configs();
        let config = load_config();
        let (saved_fan_mode, saved_fan_preset, saved_shift_mode) = load_fan_settings();
        let lang = match config.lang.as_deref() {
            Some("en") => Lang::En,
            _ => Lang::Zh,
        };
        let theme_mode = match config.theme.as_deref() {
            Some("light") => ThemeMode::Light,
            _ => ThemeMode::Dark,
        };
        let (jobs, job_rx) = mpsc::channel::<Job>();
        let (event_tx, events) = mpsc::channel::<Snapshot>();
        let _ = std::thread::Builder::new()
            .name("ecpilot-io".into())
            .spawn(move || io_worker(job_rx, event_tx));
        let mut state = Self {
            page,
            lang,
            theme_mode,
            custom: load_custom(),
            curve_target: CurveTarget::Cpu,
            status: ec::Status::default(),
            mux: "Unknown".into(),
            klc: klc::detect(),
            brightness: 255,
            color: (255, 255, 255),
            message: String::new(),
            message_ok: true,
            details: details::Details::read(),
            cpu_history: VecDeque::new(),
            gpu_history: VecDeque::new(),
            fan_history: VecDeque::new(),
            battery_history: VecDeque::new(),
            focus: cx.focus_handle(),
            focused: false,
            close_to_tray: load_close_to_tray(),
            fan_preset: saved_fan_preset.clone(),
            thermal_level_seen: None,
            last_hot_notify: None,
            jobs,
        };
        state.start_event_loop(cx, events);
        state.restore_fan_config(cx, saved_fan_mode, saved_fan_preset, saved_shift_mode);
        state
    }

    /// 启动时恢复上次选择的风扇模式、曲线预设和性能模式（后台执行）。
    fn restore_fan_config(
        &mut self,
        cx: &mut Context<Self>,
        mode: Option<String>,
        preset: Option<String>,
        shift: Option<String>,
    ) {
        let custom = self.custom.clone();
        self.run_action_quiet(cx, move || {
            let mut errors = Vec::new();
            if let Some(preset) = preset.as_deref() {
                let result = if preset == "custom" {
                    ec::apply_custom(
                        &custom.cpu_temp,
                        &custom.cpu_speed,
                        &custom.gpu_temp,
                        &custom.gpu_speed,
                    )
                } else {
                    ec::apply_preset(preset)
                };
                if let Err(error) = result {
                    errors.push(format!("{error:#}"));
                }
            }
            if let Some(mode) = mode.as_deref() {
                if let Err(error) = ec::set_fan_mode(mode) {
                    errors.push(format!("{error:#}"));
                }
            }
            if let Some(shift) = shift.as_deref() {
                if let Err(error) = ec::set_shift_mode(shift) {
                    errors.push(format!("{error:#}"));
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(anyhow::anyhow!(errors.join("; ")))
            }
        });
    }

    /// 应用后台线程采样的硬件快照。
    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        self.status = snapshot.status;
        self.mux = snapshot.mux;
        self.details.mem = snapshot.mem;
        self.details.swp = snapshot.swp;
        self.details.disks = snapshot.disks;
        seed_history(&mut self.cpu_history, self.status.cpu_temp as f32);
        seed_history(&mut self.gpu_history, self.status.gpu_util as f32);
        seed_history(&mut self.fan_history, (self.status.cpu_fan as f32 / 60.0).min(100.0));
        seed_history(&mut self.battery_history, self.status.battery as f32);

        // 温控换档与高温提醒。首次采样只记录基线，不打扰用户。
        let level = self.status.thermal_level;
        if let Some(previous) = self.thermal_level_seen {
            if previous != level {
                self.notify_thermal_level(level);
            }
        } else {
            self.thermal_level_seen = Some(level);
        }
        if self.status.tctl >= 88 {
            let due = self
                .last_hot_notify
                .is_none_or(|at| at.elapsed() >= Duration::from_secs(900));
            if due {
                self.last_hot_notify = Some(Instant::now());
                let title = self.t("CPU 高温", "High CPU temperature");
                let body = format!(
                    "{} {}°C · {}",
                    self.t("当前", "now"),
                    self.status.tctl,
                    self.t(
                        "建议开启 Cooler Boost（狂暴模式）",
                        "consider enabling Cooler Boost"
                    )
                );
                spawn_notify(title, body, "normal");
            }
        }
    }

    /// 温控换档通知。
    fn notify_thermal_level(&self, level: i32) {
        let cap = self.status.thermal_cap_mhz as f32 / 1000.0;
        let (title, body, urgency) = match level {
            0 => (
                self.t("温控：恢复正常", "Thermal: normal"),
                format!(
                    "{} {:.1} GHz · boost {}",
                    self.t("上限", "cap"),
                    cap,
                    if self.status.thermal_boost {
                        self.t("开", "on")
                    } else {
                        self.t("关", "off")
                    }
                ),
                "low",
            ),
            1 => (
                self.t("温控：降档 1", "Thermal: level 1"),
                format!("{} {:.1} GHz", self.t("上限", "cap"), cap),
                "normal",
            ),
            2 => (
                self.t("温控：降档 2", "Thermal: level 2"),
                format!(
                    "{} {:.1} GHz · boost {}",
                    self.t("上限", "cap"),
                    cap,
                    self.t("关", "off")
                ),
                "normal",
            ),
            _ => (
                self.t("温控：紧急锁存", "Thermal: emergency latch"),
                format!(
                    "{} {:.1} GHz · {}",
                    self.t("上限", "cap"),
                    cap,
                    self.t("请降低负载", "reduce the load")
                ),
                "critical",
            ),
        };
        spawn_notify(title, body, urgency);
    }

    /// 在后台线程执行特权操作；结果回到 UI 线程。
    pub(crate) fn run_action<F>(&mut self, cx: &mut Context<Self>, work: F)
    where
        F: FnOnce() -> anyhow::Result<()> + Send + 'static,
    {
        self.run_action_inner(cx, work, |_| {}, false);
    }

    /// 同上，但成功时不显示“已完成”提示（用于启动恢复）。
    pub(crate) fn run_action_quiet<F>(&mut self, cx: &mut Context<Self>, work: F)
    where
        F: FnOnce() -> anyhow::Result<()> + Send + 'static,
    {
        self.run_action_inner(cx, work, |_| {}, true);
    }

    /// 同上，并在成功时回到 UI 线程执行 `on_success`（用于更新本地状态）。
    pub(crate) fn run_action_then<F, S>(
        &mut self,
        cx: &mut Context<Self>,
        work: F,
        on_success: S,
    ) where
        F: FnOnce() -> anyhow::Result<()> + Send + 'static,
        S: FnOnce(&mut Self) + 'static,
    {
        self.run_action_inner(cx, work, on_success, false);
    }

    fn run_action_inner<F, S>(
        &mut self,
        cx: &mut Context<Self>,
        work: F,
        on_success: S,
        quiet_success: bool,
    ) where
        F: FnOnce() -> anyhow::Result<()> + Send + 'static,
        S: FnOnce(&mut Self) + 'static,
    {
        let (result_tx, result_rx) = mpsc::channel();
        let job: Job = Box::new(move || {
            let _ = result_tx.send(work());
        });
        if self.jobs.send(job).is_err() {
            self.action(Err(anyhow::anyhow!("后台 I/O 线程已停止")), cx);
            return;
        }
        cx.spawn(async move |this, cx| loop {
            match result_rx.try_recv() {
                Ok(result) => {
                    let ok = result.is_ok();
                    let _ = this.update(cx, |this, cx| {
                        if ok {
                            on_success(this);
                        }
                        if quiet_success && ok {
                            cx.notify();
                        } else {
                            this.action(result, cx);
                        }
                    });
                    break;
                }
                Err(TryRecvError::Empty) => {
                    cx.background_executor()
                        .timer(Duration::from_millis(50))
                        .await;
                }
                Err(TryRecvError::Disconnected) => {
                    let _ = this.update(cx, |this, cx| {
                        this.action(Err(anyhow::anyhow!("后台 I/O 线程已停止")), cx)
                    });
                    break;
                }
            }
        })
        .detach();
    }

    /// 消费后台快照并刷新 UI。
    fn start_event_loop(&self, cx: &mut Context<Self>, events: Receiver<Snapshot>) {
        cx.spawn(async move |this, cx| loop {
            loop {
                match events.try_recv() {
                    Ok(snapshot) => {
                        if this
                            .update(cx, |this, cx| {
                                this.apply_snapshot(snapshot);
                                cx.notify();
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
        })
        .detach();
    }

    /// 返回当前主题配色。
    pub(crate) fn theme(&self) -> Theme {
        match self.theme_mode {
            ThemeMode::Dark => DARK,
            ThemeMode::Light => LIGHT,
        }
    }

    /// 按当前语言选择中/英文文本。
    pub(crate) fn t(&self, zh: &str, en: &str) -> String {
        match self.lang {
            Lang::Zh => zh.to_string(),
            Lang::En => en.to_string(),
        }
    }

    /// 执行一个特权操作，并把结果写入顶部提示。
    pub(crate) fn action(&mut self, result: anyhow::Result<()>, cx: &mut Context<Self>) {
        let (message, ok) = match result {
            Ok(()) => (self.t("操作已完成", "Done"), true),
            Err(error) => (format!("{error:#}"), false),
        };
        self.message = message;
        self.message_ok = ok;
        cx.notify();
    }


}

impl gpui::Focusable for AppState {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus.clone()
    }
}

impl Render for AppState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focused {
            cx.focus_self(window);
            self.focused = true;
        }
        div()
            .flex()
            .flex_row()
            .size_full()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                let page = match event.keystroke.key.as_str() {
                    "1" => Some(Page::Overview),
                    "2" => Some(Page::Fan),
                    "3" => Some(Page::Keyboard),
                    "4" => Some(Page::Gpu),
                    "5" => Some(Page::Telemetry),
                    "6" => Some(Page::Thermal),
                    "7" => Some(Page::Settings),
                    _ => None,
                };
                if let Some(page) = page {
                    this.page = page;
                    cx.notify();
                }
            }))
            .bg(rgb(self.theme().bg))
            .text_color(rgb(self.theme().text))
            .text_size(px(14.0))
            .child(self.sidebar(cx))
            .child(self.content(cx))
    }
}

fn seed_history(history: &mut VecDeque<f32>, value: f32) {
    if history.is_empty() {
        for _ in 0..30 {
            history.push_back(value);
        }
    }
    push_history(history, value);
}

fn push_history(history: &mut VecDeque<f32>, value: f32) {
    history.push_back(value);
    while history.len() > 48 {
        history.pop_front();
    }
}

/// 通过 notify-send 发桌面通知；没有通知服务时静默失败。
fn spawn_notify(title: String, body: String, urgency: &str) {
    let urgency = urgency.to_owned();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("notify-send")
            .args([
                "-a",
                "ECPilot",
                "-u",
                urgency.as_str(),
                "-i",
                "dialog-warning",
                "--",
                title.as_str(),
                body.as_str(),
            ])
            .status();
    });
}
