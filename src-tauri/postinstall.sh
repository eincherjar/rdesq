#!/bin/sh
gtk-update-icon-cache /usr/share/icons/hicolor/ >/dev/null 2>&1 || :
update-desktop-database >/dev/null 2>&1 || :
# KDE cache rebuild — needed for icon to appear in launcher/taskbar
kbuildsycoca6 >/dev/null 2>&1 || kbuildsycoca5 >/dev/null 2>&1 || true
