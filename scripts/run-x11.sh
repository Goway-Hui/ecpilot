#!/bin/sh
# FluxDown-style X11 launch: hide WAYLAND_DISPLAY so GPUI uses X11/XWayland.
exec env -u WAYLAND_DISPLAY ecpilot "$@"
