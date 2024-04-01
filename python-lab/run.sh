#!/bin/bash

# 目标目录
TARGET_DIR="./alpine"
# 文件名和下载地址
FILE_NAME="rootfs-alpine.tgz"
DOWNLOAD_URL="https://cloud.tsinghua.edu.cn/f/8da0e9ab50d4489f870e/?dl=1"
QCOW2_FILE="rootfs-alpine.qcow2"

# 检查目录是否存在，不存在则创建
if [ ! -d "$TARGET_DIR" ]; then
    mkdir -p "$TARGET_DIR"
fi

cd "$TARGET_DIR"

# 检查文件是否存在
if [ ! -f "$QCOW2_FILE" ]; then
    echo "文件 ${QCOW2_FILE} 不存在，开始下载..."
    wget -O "$FILE_NAME" "$DOWNLOAD_URL"
    echo "正在解压文件..."
    tar -zxvf "$FILE_NAME"
fi

# 判断文件是否成功下载
if [ ! -f "$QCOW2_FILE" ]; then
    echo "下载失败，请检查URL或网络设置。"
    exit 1
fi

# 启动QEMU
echo "启动QEMU..."
qemu-system-aarch64 -m 512 -cpu cortex-a53 -M virt -serial mon:stdio -nographic \
-bios /usr/share/qemu-efi-aarch64/QEMU_EFI.fd \
-netdev user,hostfwd=tcp::2222-:22,id=eth0 -device virtio-net-device,netdev=eth0 \
-hda "$QCOW2_FILE"