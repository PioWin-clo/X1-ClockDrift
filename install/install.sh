#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/home/x1pio/strontium-meter"
REPO_DIR="$INSTALL_DIR/repo"
BIN_DIR="$INSTALL_DIR/bin"
FRONTEND_DIR="$INSTALL_DIR/frontend"
KEY_PATH="/home/x1pio/.ssh/x1cd_deploy_key"
GH_REPO="git@github.com:PioWin-clo/x1-clockdrift.git"
BIN_NAME="x1cd"
BIN_URL="https://github.com/PioWin-clo/x1-clockdrift/releases/latest/download/x1cd-linux-x86_64"
SERVICE_FILE_LOCAL="$(dirname "$0")/strontium-meter.service"
SERVICE_FILE_DST="/etc/systemd/system/x1cd.service"

if [ "$(id -un)" != "x1pio" ]; then
  echo "Run as user x1pio (got: $(id -un))" >&2
  exit 1
fi

echo "==> Creating directories at $INSTALL_DIR"
mkdir -p "$BIN_DIR" "$REPO_DIR" "$FRONTEND_DIR"
chmod 0755 "$INSTALL_DIR"

if [ -f "$BIN_DIR/$BIN_NAME" ] && [ "${SKIP_DOWNLOAD:-0}" = "1" ]; then
  echo "==> Skipping binary download (SKIP_DOWNLOAD=1)"
else
  echo "==> Downloading $BIN_NAME from $BIN_URL"
  if command -v curl >/dev/null 2>&1; then
    curl -fL --retry 3 -o "$BIN_DIR/$BIN_NAME" "$BIN_URL"
  else
    wget -O "$BIN_DIR/$BIN_NAME" "$BIN_URL"
  fi
  chmod +x "$BIN_DIR/$BIN_NAME"
fi

echo "==> Generating deploy key (if needed)"
mkdir -p "$(dirname "$KEY_PATH")"
chmod 0700 "$(dirname "$KEY_PATH")"
if [ ! -f "$KEY_PATH" ]; then
  ssh-keygen -t ed25519 -f "$KEY_PATH" -N "" -C "x1cd-deploy@$(hostname)"
  echo
  echo "===================================================="
  echo "DEPLOY KEY generated. Public key:"
  cat "${KEY_PATH}.pub"
  echo "===================================================="
  echo "Add it at: https://github.com/PioWin-clo/x1-clockdrift/settings/keys/new"
  echo "with 'Allow write access' enabled."
  echo
  read -r -p "Press Enter once the deploy key is registered… " _
fi

echo "==> Cloning repo to $REPO_DIR"
if [ ! -d "$REPO_DIR/.git" ]; then
  GIT_SSH_COMMAND="ssh -i $KEY_PATH -o StrictHostKeyChecking=accept-new" \
    git clone -b data "$GH_REPO" "$REPO_DIR" || \
    GIT_SSH_COMMAND="ssh -i $KEY_PATH -o StrictHostKeyChecking=accept-new" \
    git clone "$GH_REPO" "$REPO_DIR"
  cd "$REPO_DIR"
  git checkout data 2>/dev/null || git checkout -B data
  git config user.email "x1cd@$(hostname)"
  git config user.name "x1cd-bot"
  cd - >/dev/null
fi

echo "==> Copying frontend files into $FRONTEND_DIR"
if [ -d "$(dirname "$0")/../frontend" ]; then
  cp "$(dirname "$0")/../frontend/"*.html "$FRONTEND_DIR/"
  cp "$(dirname "$0")/../frontend/"*.css "$FRONTEND_DIR/"
  cp "$(dirname "$0")/../frontend/"*.js "$FRONTEND_DIR/"
  cp "$(dirname "$0")/../frontend/"*.html "$REPO_DIR/" || true
  cp "$(dirname "$0")/../frontend/"*.css "$REPO_DIR/" || true
  cp "$(dirname "$0")/../frontend/"*.js "$REPO_DIR/" || true
fi

echo "==> Writing config.toml"
cat > "$INSTALL_DIR/config.toml" <<TOML
log_path = "/home/x1pio/validator.log"
rpc_url = "http://localhost:8899"
db_path = "$INSTALL_DIR/data.db"
api_listen = "127.0.0.1:8088"
git_repo_path = "$REPO_DIR"
git_remote_url = "$GH_REPO"
git_deploy_key = "$KEY_PATH"
git_branch = "data"
export_interval_secs = 300
rpc_rate_limit_per_sec = 5
retention_days = 30
kill_switch_path = "$INSTALL_DIR/STOP"
watchdog_secs = 120
stake_refresh_secs = 3600
history_retention_days = 7
frontend_dir = "$FRONTEND_DIR"
TOML

echo "==> Installing systemd unit (sudo required)"
sudo install -m 0644 "$SERVICE_FILE_LOCAL" "$SERVICE_FILE_DST"
sudo systemctl daemon-reload
sudo systemctl enable x1cd

echo
echo "Install done."
echo "Start:    sudo systemctl start x1cd"
echo "Status:   systemctl status x1cd"
echo "Logs:     journalctl -u x1cd -f"
echo "Stop:     touch $INSTALL_DIR/STOP   (or 'sudo systemctl stop x1cd')"
