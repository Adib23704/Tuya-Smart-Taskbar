#!/usr/bin/env bash
set -euo pipefail

echo "This script installs common system dependencies required to build/run Tuya Smart Taskbar on Xubuntu (Ubuntu/Debian)."
echo "Review the commands before running. Run with sudo when prompted."

if [ "$EUID" -ne 0 ]; then
  echo "Note: Some commands below require sudo. The script will invoke sudo where necessary."
fi

echo "Updating apt and installing system packages..."
sudo apt update
sudo apt install -y build-essential curl git pkg-config libssl-dev libgtk-3-dev libayatana-appindicator3-dev libwebkit2gtk-4.0-dev

echo "Installing Node.js (18.x) via NodeSource..."
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

echo "Enabling corepack and activating pnpm ${PNPM_VERSION:-10.24.0} (project expects pnpm@10.24.0)..."
corepack enable || true
corepack prepare pnpm@10.24.0 --activate || sudo npm i -g pnpm@10.24.0

if ! command -v rustc >/dev/null 2>&1; then
  echo "Installing Rust (rustup)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "Done. From the project root run:"
echo "  pnpm install"
echo "  pnpm dev    # development"
echo "  pnpm build  # production build"

echo "If you see errors about missing webkit or GTK headers, ensure 'libwebkit2gtk-4.0-dev' and 'libgtk-3-dev' are installed."

exit 0
