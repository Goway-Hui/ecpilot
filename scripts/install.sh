#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
PREFIX="${PREFIX:-/usr/local}"

# 旧 MControlCenter 命名的迁移清理。
systemctl stop mcontrolcenter-gpui-helper 2>/dev/null || true
systemctl disable mcontrolcenter-gpui-helper 2>/dev/null || true
rm -f "$PREFIX/bin/mcontrolcenter-gpui" "$PREFIX/libexec/mcontrolcenter-gpui-helper"
rm -f /etc/systemd/system/mcontrolcenter-gpui-helper.service
rm -f /etc/dbus-1/system.d/mcontrolcenter-gpui-helper.conf
rm -f /usr/share/applications/mcontrolcenter-gpui.desktop /usr/share/applications/mcontrolcenter-gpui-wayland.desktop
if [ -f /var/lib/mcontrolcenter-gpui/thermal-enabled ] && [ ! -f /var/lib/ecpilot/thermal-enabled ]; then
    mkdir -p /var/lib/ecpilot
    cp /var/lib/mcontrolcenter-gpui/thermal-enabled /var/lib/ecpilot/thermal-enabled
fi

install -Dm755 target/release/ecpilot "$PREFIX/bin/ecpilot"
install -Dm755 target/release/ecpilot-helper "$PREFIX/libexec/ecpilot-helper"
install -Dm644 packaging/ecpilot-helper.service /etc/systemd/system/
install -Dm644 packaging/ecpilot-helper.conf /etc/dbus-1/system.d/
install -Dm644 packaging/ecpilot.desktop /usr/share/applications/
install -Dm644 packaging/ecpilot-wayland.desktop /usr/share/applications/
install -Dm644 assets/icons/ecpilot.svg \
    "$PREFIX/share/icons/hicolor/scalable/apps/ecpilot.svg"
for size in 16 24 32 48 64 128 256; do
    install -Dm644 "assets/icons/ecpilot-$size.png" \
        "$PREFIX/share/icons/hicolor/${size}x${size}/apps/ecpilot.png"
done
gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
xdg-icon-resource forceupdate --theme hicolor 2>/dev/null || true
systemctl daemon-reload
systemctl reload dbus.service 2>/dev/null || true
systemctl enable --now ecpilot-helper
