//! 概览页使用的系统信息（fastfetch 风格）：静态信息只读一次，动态信息定时刷新。

use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Default)]
pub struct Details {
    pub os: String,
    pub kernel: String,
    pub packages: String,
    pub age: String,
    pub user: String,
    pub wm: String,
    pub shell: String,
    pub term: String,
    pub font: String,
    pub pc: String,
    pub cpu: String,
    pub mem: String,
    pub swp: String,
    pub gpu_amd: String,
    pub mon: String,
    pub disks: Vec<String>,
}

impl Details {
    pub fn read() -> Self {
        let mut details = Self {
            os: os_name(),
            kernel: read_trim("/proc/sys/kernel/osrelease"),
            packages: package_count(),
            age: system_age(),
            user: user_at_host(),
            wm: window_manager(),
            shell: shell_name(),
            term: terminal_name(),
            font: crate::theme::FONT_FAMILY.to_string(),
            pc: pc_name(),
            cpu: cpu_summary(),
            mem: String::new(),
            swp: String::new(),
            gpu_amd: amd_gpu_summary(),
            mon: monitor_summary(),
            disks: Vec::new(),
        };
        details.refresh_dynamic();
        details
    }

    /// Refresh the values that change while the app runs.
    pub fn refresh_dynamic(&mut self) {
        let dynamic = read_dynamic();
        self.mem = dynamic.mem;
        self.swp = dynamic.swp;
        self.disks = dynamic.disks;
    }
}

/// 需要周期刷新、且可能阻塞的字段；在后台线程读取。
pub(crate) struct Dynamic {
    pub mem: String,
    pub swp: String,
    pub disks: Vec<String>,
}

pub(crate) fn read_dynamic() -> Dynamic {
    Dynamic {
        mem: memory_summary(),
        swp: swap_summary(),
        disks: disk_summaries(),
    }
}

fn read_trim(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn os_name() -> String {
    let mut pretty = String::new();
    if let Ok(text) = fs::read_to_string("/etc/os-release") {
        for line in text.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                pretty = value.trim_matches('"').to_string();
                break;
            }
        }
    }
    if pretty.is_empty() {
        pretty = "Linux".to_string();
    }
    format!("{pretty} {}", std::env::consts::ARCH)
}

fn package_count() -> String {
    fs::read_dir("/var/lib/pacman/local")
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_dir())
                .count()
                .to_string()
        })
        .unwrap_or_else(|_| "—".to_string())
}

fn system_age() -> String {
    let created = fs::metadata("/")
        .and_then(|metadata| metadata.created())
        .or_else(|_| fs::metadata("/var/lib/pacman").and_then(|metadata| metadata.created()));
    match created {
        Ok(time) => {
            let days = SystemTime::now()
                .duration_since(time)
                .map(|duration| duration.as_secs() / 86_400)
                .unwrap_or(0);
            match days {
                0 => "today".to_string(),
                1 => "1 day".to_string(),
                n => format!("{n} days"),
            }
        }
        Err(_) => "—".to_string(),
    }
}

fn user_at_host() -> String {
    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let host = read_trim("/etc/hostname");
    if host.is_empty() {
        user
    } else {
        format!("{user}@{host}")
    }
}

fn window_manager() -> String {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".to_string());
    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    if session.is_empty() {
        desktop
    } else {
        format!("{desktop} ({session})")
    }
}

fn shell_name() -> String {
    if let Some(name) = std::env::var("SHELL")
        .ok()
        .and_then(|path| Path::new(&path).file_name().map(|name| name.to_string_lossy().into_owned()))
    {
        return name;
    }
    let user = std::env::var("USER").unwrap_or_default();
    if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 7 && fields[0] == user {
                return fields[6].rsplit('/').next().unwrap_or("unknown").to_string();
            }
        }
    }
    "unknown".to_string()
}

fn terminal_name() -> String {
    std::env::var("TERM_PROGRAM")
        .or_else(|_| std::env::var("TERM"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn pc_name() -> String {
    let product = read_trim("/sys/class/dmi/id/product_name");
    let version = read_trim("/sys/class/dmi/id/product_version");
    if product.is_empty() {
        "Unknown PC".to_string()
    } else if version.is_empty() {
        product
    } else {
        format!("{product} ({version})")
    }
}

fn cpu_summary() -> String {
    let mut model = String::new();
    let mut cores = 0usize;
    if let Ok(text) = fs::read_to_string("/proc/cpuinfo") {
        for line in text.lines() {
            if line.starts_with("processor") {
                cores += 1;
            } else if model.is_empty() {
                if let Some(value) = line.strip_prefix("model name") {
                    model = value
                        .split(':')
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                }
            }
        }
    }
    if model.is_empty() {
        model = "CPU".to_string();
    }

    let mut frequency = String::new();
    if let Ok(text) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        if let Ok(khz) = text.trim().parse::<f64>() {
            frequency = format!(" @ {:.2} GHz", khz / 1_000_000.0);
        }
    }
    format!("{model} ({cores}){frequency}")
}

fn memory_summary() -> String {
    let (total, available) = meminfo_pair("MemTotal", "MemAvailable");
    if total == 0 {
        return "—".to_string();
    }
    let used = total.saturating_sub(available);
    format!(
        "{:.2} GiB / {:.2} GiB ({:.0}%)",
        used as f64 / 1_048_576.0,
        total as f64 / 1_048_576.0,
        used as f64 / total as f64 * 100.0
    )
}

fn swap_summary() -> String {
    let (total, free) = meminfo_pair("SwapTotal", "SwapFree");
    if total == 0 {
        return "—".to_string();
    }
    let used = total.saturating_sub(free);
    format!(
        "{:.2} GiB / {:.2} GiB ({:.0}%)",
        used as f64 / 1_048_576.0,
        total as f64 / 1_048_576.0,
        used as f64 / total as f64 * 100.0
    )
}

fn meminfo_pair(total_key: &str, free_key: &str) -> (u64, u64) {
    let mut total = 0u64;
    let mut free = 0u64;
    if let Ok(text) = fs::read_to_string("/proc/meminfo") {
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let Some(key) = parts.next() else { continue };
            let Some(value) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
                continue;
            };
            match key.trim_end_matches(':') {
                key if key == total_key => total = value,
                key if key == free_key => free = value,
                _ => {}
            }
        }
    }
    (total, free)
}

fn amd_gpu_summary() -> String {
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            if read_trim(&path.join("name").to_string_lossy()) != "amdgpu" {
                continue;
            }
            if let Ok(text) = fs::read_to_string(path.join("temp1_input")) {
                if let Ok(milli) = text.trim().parse::<f64>() {
                    return format!("AMD Radeon iGPU {:.1}°C", milli / 1000.0);
                }
            }
        }
    }
    "AMD Radeon iGPU".to_string()
}

fn monitor_summary() -> String {
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.contains("eDP") {
                continue;
            }
            let path = entry.path();
            if read_trim(&path.join("status").to_string_lossy()) != "connected" {
                continue;
            }
            let mode = read_trim(&path.join("modes").to_string_lossy())
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            if !mode.is_empty() {
                return format!("{name} {mode}");
            }
        }
    }
    "—".to_string()
}

fn disk_summaries() -> Vec<String> {
    use nix::sys::statvfs::statvfs;

    let mut disks = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let Ok(mounts) = fs::read_to_string("/proc/mounts") else {
        return disks;
    };
    for line in mounts.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 3 {
            continue;
        }
        let mount = fields[1].replace("\\040", " ");
        let fs_type = fields[2];
        if fs_type != "btrfs" && fs_type != "fuseblk" {
            continue;
        }
        let Ok(stat) = statvfs(mount.as_str()) else {
            continue;
        };
        let total = stat.blocks() * stat.fragment_size();
        let free = stat.blocks_available() * stat.fragment_size();
        if total == 0 {
            continue;
        }
        if !seen.insert((fs_type.to_string(), total)) {
            continue;
        }
        let used = total.saturating_sub(free);
        disks.push(format!(
            "{} / {} ({:.0}%) - {}",
            human_bytes(used),
            human_bytes(total),
            used as f64 / total as f64 * 100.0,
            fs_type
        ));
        if disks.len() == 2 {
            break;
        }
    }
    disks
}

fn human_bytes(bytes: u64) -> String {
    const GIB: f64 = 1_073_741_824.0;
    const TIB: f64 = 1_099_511_627_776.0;
    let value = bytes as f64;
    if value >= TIB {
        format!("{:.2} TiB", value / TIB)
    } else {
        format!("{:.2} GiB", value / GIB)
    }
}
