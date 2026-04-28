#!/usr/bin/env bash
set -euo pipefail

KEY_PATH="${1:-/home/x1pio/.ssh/x1cd_deploy_key}"

if [ -f "$KEY_PATH" ]; then
  echo "Key already exists at $KEY_PATH"
  echo "Public key:"
  cat "${KEY_PATH}.pub"
  exit 0
fi

mkdir -p "$(dirname "$KEY_PATH")"
chmod 0700 "$(dirname "$KEY_PATH")"
ssh-keygen -t ed25519 -f "$KEY_PATH" -N "" -C "x1cd-deploy@$(hostname)"

echo
echo "===================================================="
echo "Deploy key written to $KEY_PATH"
echo "===================================================="
echo "Public key (add to GitHub repo settings → Deploy keys):"
cat "${KEY_PATH}.pub"
echo
echo "URL: https://github.com/PioWin-clo/x1-clockdrift/settings/keys/new"
echo "Tick 'Allow write access'."
