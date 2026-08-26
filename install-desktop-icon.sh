#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPIMAGE_SRC="$SCRIPT_DIR/One-Click-Media-Downloader.AppImage"
ICON_SRC="$SCRIPT_DIR/src-tauri/icons/128x128@2x.png"

echo "=========================================================="
echo " Cài đặt One-Click Media Downloader vào Menu Ứng dụng Ubuntu "
echo "=========================================================="

if [ ! -f "$APPIMAGE_SRC" ]; then
    echo "Lỗi: Không tìm thấy $APPIMAGE_SRC"
    exit 1
fi

chmod +x "$APPIMAGE_SRC"

# 1. Thư mục đích tiêu chuẩn người dùng
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

# 2. Cài đặt icon
cp "$ICON_SRC" "$ICON_DIR/one-click-media-downloader.png"
# Copy thêm vào thư mục icons chung để tương thích tối đa
mkdir -p "$HOME/.local/share/icons"
cp "$ICON_SRC" "$HOME/.local/share/icons/one-click-media-downloader.png"

# 3. Tạo file .desktop cho menu Ubuntu
DESKTOP_FILE="$DESKTOP_DIR/one-click-media-downloader.desktop"

cat <<EOF > "$DESKTOP_FILE"
[Desktop Entry]
Name=One-Click Media Downloader
GenericName=Media Downloader
Comment=Tải video & âm thanh 4K/HDR, TikTok, YouTube, SoundCloud 1-Click
Exec="$APPIMAGE_SRC" --appimage-extract-and-run %U
Icon=one-click-media-downloader
Terminal=false
Type=Application
Categories=AudioVideo;Audio;Video;Network;
StartupNotify=true
StartupWMClass=one-click-media-downloader
MimeType=x-scheme-handler/ocmd;
EOF

chmod +x "$DESKTOP_FILE"

# 4. Nếu có thư mục Desktop (Bàn làm việc), tạo thêm lối tắt Desktop
if [ -d "$HOME/Desktop" ]; then
    cp "$DESKTOP_FILE" "$HOME/Desktop/"
    chmod +x "$HOME/Desktop/one-click-media-downloader.desktop"
    gio set "$HOME/Desktop/one-click-media-downloader.desktop" metadata::trusted true 2>/dev/null || true
    echo "✓ Đã tạo lối tắt trên màn hình Desktop: $HOME/Desktop/one-click-media-downloader.desktop"
fi

# 5. Cập nhật cơ sở dữ liệu menu ứng dụng Ubuntu
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo "=========================================================="
echo "✓ THÀNH CÔNG! Ứng dụng đã xuất hiện trong Menu Ubuntu."
echo "Bạn có thể:"
echo "1. Nhấn phím Super (phím Windows), gõ 'Media Downloader' và click vào icon để mở app."
echo "2. Hoặc click đúp vào icon trên màn hình Desktop."
echo "3. Hoàn toàn không cần mở terminal!"
echo "=========================================================="
