# ECPilot

A Rust + GPUI MSI laptop control center for Linux, with
SteelSeries KLC per-key RGB support built in.

This is an independent project; it does not modify or depend on the original
Qt-based `MControlCenter` working tree.

## Features

- Live overview: CPU/GPU temperature, fan RPM, battery, charge limit, EC
  firmware, fan/shift mode
- Fan control: auto / silent / advanced, four curve presets (default, quiet,
  gaming, max), per-point curve editor, Cooler Boost. Fan mode, curve preset,
  custom curve and shift mode are saved and re-applied at startup
- Built-in thermal guard: Tctl/Tccd1/Tccd2, current level/cap/boost, enable
  or disable the helper-side guard, and restart it from the Thermal page
- Shift mode: Eco / Comfort / Turbo
- Battery charge limit: 60% / 80% / 100%
- Webcam on/off, USB Power Share on/off, Fn/Win swap
- Per-key RGB: solid colors, software brightness, all-off (SteelSeries KLC
  `1038:113a` and the wider KLC family)
- GPU MUX: Hybrid/Discrete status and switching
- GPU telemetry: power, utilization, temperature, VRAM (cached for 2 s)
- Chinese/English UI switch; dark/light theme; language, theme and custom
  fan curve are persisted under `$XDG_CONFIG_HOME/ecpilot/`
- Single-instance guard and a StatusNotifierItem tray: left click (or the
  "显示窗口" menu item) restores the window; closing minimizes to the tray by
  default, and the tray menu can quit the whole app. GPUI has no native tray API, so `ksni` runs its own D-Bus
  thread and sends `TrayCommand`s to the GPUI main thread.

## Architecture

| Path | Purpose |
|---|---|
| `src/main.rs` | GPUI views, navigation, language, single-instance lock, startup policy |
| `src/ec.rs` | unprivileged sysfs reads; zbus client for the root helper; 2 s GPU/EC caches |
| `src/helper.rs` | root D-Bus helper: EC whitelist, batched writes, sysfs switches |
| `src/bin/helper.rs` | standalone `ecpilot-helper` binary |
| `src/tray.rs` | minimal ksni StatusNotifierItem |
| `src/klc/` | KLC protocol, hidraw I/O, model/layout tables (ported from `msi-klc`) |
| `packaging/ecpilot-helper.service` | root helper systemd unit |
| `packaging/ecpilot-helper.conf` | D-Bus policy (root + wheel) |
| `packaging/PKGBUILD` | Arch package template |
| `scripts/install.sh` | local `/usr/local` installer |

Privilege model:

- The GUI runs as the desktop user. The default launch is X11, matching
  FluxDown's local desktop entry: `env -u WAYLAND_DISPLAY ecpilot`.
  Wayland is opt-in with `MCC_ALLOW_WAYLAND=1`.
- EC fan mode, curves, shift mode, battery limit, webcam, USB Power Share and
  Fn/Win swap are sent to the root `ecpilot.helper` system D-Bus
  service. The helper accepts only a whitelist of EC addresses (fan, curve,
  cooler, shift, USB, Fn, backlight) and rejects arbitrary writes.
- Curve writes use one batched `putValues` call instead of 26 round trips.
- The D-Bus policy allows calls only from `root` and the `wheel` group.
- KLC RGB writes and GPU MUX switching use `pkexec <self> --helper ...`.
- The UI itself never runs as root.

## Requirements

- Rust 1.85+ (edition 2024), Vulkan, Wayland/X11, fontconfig
- `msi-ec` loaded (`/sys/devices/platform/msi-ec`)
- `ec_sys write_support=1` for fan curves
- Optional: `/usr/local/bin/msi-gpu-switcher` for MUX switching
- `polkit` agent for KLC RGB and MUX actions
- `wheel` group membership for the helper D-Bus policy

## Build and run

```sh
cargo build --release
./target/release/ecpilot
# open a specific page:
./target/release/ecpilot --page fan
# pages: overview, fan, keyboard, gpu, telemetry, settings
```

Install the GUI and root helper:

```sh
sudo scripts/install.sh
```

The thermal guard is built into `ecpilot-helper`: it samples
`k10temp` (Tctl/Tccd1/Tccd2, using the hottest value) every 200 ms and applies
the multi-level CPU cap directly via cpufreq/EPP/boost. No external
`thermal-guard` script or systemd unit is required. Enable, disable or restart
it from the **温控 / Thermal** page; disabling applies the safe state (base
frequency, boost off). The enabled state and parameters persist in
`/var/lib/ecpilot/`; thresholds/caps can be overridden with
`MCC_THERMAL_*` environment variables on the helper service.

Uninstall the app and helper:

```sh
sudo scripts/uninstall.sh
sudo scripts/uninstall.sh --purge   # also remove user config
```

The standalone helper is `target/release/ecpilot-helper`
(about 4 MB); the GUI binary still accepts `--dbus-helper` for compatibility.

### Startup on niri

The binary defaults to **X11** (FluxDown-style): it removes `WAYLAND_DISPLAY`
and runs on X11/XWayland. `--x11` forces the same path; `MCC_ALLOW_WAYLAND=1`
keeps Wayland.

## KLC provenance

`src/klc/` is ported from
[`msi-klc`](https://github.com/echtzeit-solutions/msi-wmi-platform-dkms/tree/master/keyboard-rgb/msi-klc)
(MIT OR Apache-2.0). The original license files are kept in `src/klc/`.
The model/layout JSON files are embedded at compile time.

## Known limitations

- The X11 backend depends on niri's xwayland-satellite. If no X11 window
  appears, restart the niri session after a compositor update.
- The runtime is `gpui-fast` (FluxDown's GPUI fork), not upstream `gpui`.
- "Minimize to tray on close" is on by default: niri's `close-window`
  shortcut and the window close button minimize to the tray, so the tray
  stays available. Disable it in Settings if you want closing to quit the
  whole app (the tray disappears with the process).
- Keyboard backlight (EC 0xD3/0xF3) is not exposed on this model; KLC RGB
  controls the keyboard lighting instead.
- MUX switching delegates to `msi-gpu-switcher`.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
