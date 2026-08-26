#!/bin/bash
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPIMAGE="$DIR/src-tauri/target/release/bundle/appimage/One-Click Media Downloader_1.0.0_amd64.AppImage"
BINARY="$DIR/src-tauri/target/release/one-click-media-downloader"

chmod +x "$APPIMAGE" 2>/dev/null
chmod +x "$BINARY" 2>/dev/null

echo "=== Đang khởi chạy One-Click Media Downloader ==="

# 1. Nếu có biến môi trường DISPLAY hoặc WAYLAND_DISPLAY
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
    echo "Cảnh báo: Không phát hiện màn hình đồ họa (DISPLAY/WAYLAND_DISPLAY). Hãy chạy lệnh này từ giao diện Desktop của Ubuntu."
fi

# 2. Ưu tiên chạy trực tiếp binary release native (nhanh nhất và không phụ thuộc FUSE)
if [ -f "$BINARY" ]; then
    echo "Khởi chạy ứng dụng native..."
    exec "$BINARY" "$@"
fi

# 3. Fallback sang AppImage
if [ -f "$APPIMAGE" ]; then
    exec "$APPIMAGE" --appimage-extract-and-run "$@"
fi

echo "Lỗi: Không tìm thấy file chạy ứng dụng!"
exit 1
