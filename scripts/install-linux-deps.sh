#!/usr/bin/env bash
# Install Tauri 2 Linux build dependencies (Fedora/Nobara or Debian/Ubuntu).
set -euo pipefail

if command -v dnf >/dev/null 2>&1; then
  echo "Installing via dnf (Fedora/Nobara)…"
  sudo dnf install -y \
    webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    patchelf
  exit 0
fi

if command -v apt-get >/dev/null 2>&1; then
  echo "Installing via apt (Debian/Ubuntu)…"
  sudo apt-get update
  sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev
  exit 0
fi

echo "Unsupported distro: install webkit2gtk-4.1 dev packages manually." >&2
exit 1
