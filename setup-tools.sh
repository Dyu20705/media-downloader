#!/usr/bin/env bash
set -e

echo "=========================================================="
echo " Cài đặt các công cụ hệ thống tối ưu cho opendownloader  "
echo " (ffmpeg, mediainfo, yt-dlp, libfuse2)                   "
echo "=========================================================="
echo "Đang yêu cầu quyền quản trị sudo để cài đặt..."

sudo apt update
sudo apt install -y ffmpeg mediainfo yt-dlp libfuse2

echo ""
echo "✓ Đã cài đặt hoàn tất các công cụ hệ thống cần thiết!"
echo "Ứng dụng opendownloader sẽ tự động nhận diện các công cụ này ngay lập tức."
