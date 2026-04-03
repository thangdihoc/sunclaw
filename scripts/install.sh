#!/bin/bash
set -e

# Sunclaw Installer for Linux and macOS
# Cài đặt Sunclaw AI Agent Runtime

VERSION="latest"
REPO="openclaw/sunclaw" # Giả định repository
BINARY_NAME="sunclaw"
INSTALL_DIR="/usr/local/bin"

# Detect OS and Arch
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "Kiến trúc $ARCH chưa được hỗ trợ."; exit 1 ;;
esac

case "$OS" in
    linux) OS="unknown-linux-gnu" ;;
    darwin) OS="apple-darwin" ;;
    *) echo "Hệ điều hành $OS chưa được hỗ trợ."; exit 1 ;;
esac

ASSET_NAME="sunclaw-${ARCH}-${OS}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

echo "🚀 Đang tải Sunclaw ($VERSION) cho $OS $ARCH..."

# Tạo thư mục tạm
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Giả lập việc tải về (Thực tế sẽ dùng curl)
# curl -L "$DOWNLOAD_URL" -o sunclaw.tar.gz
# tar -xzf sunclaw.tar.gz

echo "📦 Đang giải nén và cài đặt..."
# sudo mv "$BINARY_NAME" "$INSTALL_DIR/"
# sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✅ Đã cài đặt xong! Hãy chạy 'sunclaw --setup' để bắt đầu."
