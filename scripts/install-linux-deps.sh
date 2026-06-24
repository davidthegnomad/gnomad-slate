#!/usr/bin/env bash
# Install native build dependencies for Gnomad Slate on Fedora / Nobara / RHEL-like distros.
set -euo pipefail

if ! command -v dnf >/dev/null 2>&1; then
  echo "This script targets Fedora/Nobara (dnf). For Debian/Ubuntu use:"
  echo "  sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf"
  exit 1
fi

echo "Installing Gnomad Slate Linux build dependencies..."
sudo dnf install -y \
  webkit2gtk4.1-devel \
  libsoup3-devel \
  javascriptcoregtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  openssl-devel \
  pkg-config \
  gcc \
  gcc-c++ \
  make \
  perl-FindBin

echo "Done. Also install Rust (rustup) and Node.js LTS, then:"
echo "  npm install && npm run tauri build"
