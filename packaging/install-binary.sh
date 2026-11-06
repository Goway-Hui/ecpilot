#!/bin/sh
set -eu
# 从解压后的二进制包安装到 /usr/local（需 root）。
PREFIX="${PREFIX:-/usr/local}"
HERE=$(cd "$(dirname "$0")" && pwd)

install -Dm755 "$HERE/bin/ecpilot" "$PREFIX/bin/ecpilot"
install -Dm755 "$HERE/bin/ecpilot-helper" "$PREFIX/libexec/ecpilot-helper"
install -Dm644 "$HERE/share/applications/ecpilot.desktop" /usr/share/applications/ecpilot.desktop
install -Dm644 "$HERE/share/applications/ecpilot-wayland.desktop" /usr/share/applications/ecpilot-wayland.desktop
install -Dm644 "$HERE/lib/systemd/system/ecpilot-helper.service" /etc/systemd/system/ecpilot-helper.service
install -Dm644 "$HERE/share/dbus-1/system.d/ecpilot-helper.conf" /etc/dbus-1/system.d/ecpilot-helper.conf
install -Dm644 "$HERE/share/icons/hicolor/scalable/apps/ecpilot.svg" "$PREFIX/share/icons/hicolor/scalable/apps/ecpilot.svg"
for size in 16 24 32 48 64 128 256; do
    install -Dm644 "$HERE/share/icons/hicolor/${size}x${size}/apps/ecpilot.png" \
        "$PREFIX/share/icons/hicolor/${size}x${size}/apps/ecpilot.png"
done
gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
xdg-icon-resource forceupdate --theme hicolor 2>/dev/null || true
systemctl daemon-reload
systemctl reload dbus.service 2>/dev/null || true
systemctl enable --now ecpilot-helper
echo "ECPilot installed under $PREFIX"
