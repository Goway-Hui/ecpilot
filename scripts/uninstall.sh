#!/bin/sh
set -eu

PREFIX="${PREFIX:-/usr/local}"

systemctl stop ecpilot-helper 2>/dev/null || true
systemctl disable ecpilot-helper 2>/dev/null || true

rm -f "$PREFIX/bin/ecpilot"
rm -f "$PREFIX/libexec/ecpilot-helper"
rm -f /etc/systemd/system/ecpilot-helper.service
rm -f /etc/dbus-1/system.d/ecpilot-helper.conf
rm -f /usr/share/applications/ecpilot.desktop
rm -f /usr/share/applications/ecpilot-wayland.desktop
rm -rf /var/lib/ecpilot

# 清理旧 MControlCenter 命名残留。
systemctl stop mcontrolcenter-gpui-helper 2>/dev/null || true
systemctl disable mcontrolcenter-gpui-helper 2>/dev/null || true
rm -f "$PREFIX/bin/mcontrolcenter-gpui" "$PREFIX/libexec/mcontrolcenter-gpui-helper"
rm -f /etc/systemd/system/mcontrolcenter-gpui-helper.service
rm -f /etc/dbus-1/system.d/mcontrolcenter-gpui-helper.conf
rm -f /usr/share/applications/mcontrolcenter-gpui.desktop /usr/share/applications/mcontrolcenter-gpui-wayland.desktop
rm -rf /var/lib/mcontrolcenter-gpui

systemctl daemon-reload

if [ "${1:-}" = "--purge" ]; then
    # sudo 下 HOME 可能指向 /root；用 SUDO_USER/UID 找真正的调用者。
    if [ -n "${SUDO_USER:-}" ]; then
        user_home="$(getent passwd "${SUDO_USER}" | cut -d: -f6)"
    else
        user_home="${HOME:-}"
    fi
    if [ -n "${user_home}" ]; then
        rm -rf "${XDG_CONFIG_HOME:-${user_home}/.config}/ecpilot"
    fi
fi

echo "ECPilot removed. User config kept unless --purge was given."
