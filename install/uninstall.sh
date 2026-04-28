#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/home/x1pio/strontium-meter"

echo "Stopping and disabling service…"
sudo systemctl stop x1cd 2>/dev/null || true
sudo systemctl disable x1cd 2>/dev/null || true
sudo rm -f /etc/systemd/system/x1cd.service
sudo systemctl daemon-reload

echo "Removing $INSTALL_DIR (deploy key and repo retained)"
read -r -p "Also remove $INSTALL_DIR including db and repo? [y/N] " ans
case "$ans" in
  y|Y) rm -rf "$INSTALL_DIR" ;;
  *) echo "Keeping $INSTALL_DIR" ;;
esac

echo "Done."
