#!/bin/bash

PYTHON_BIN_PATH="python-lab/opt/python3.11/bin/python3.11"

if [ -f "disk.img" ]; then
    rm disk.img
fi

if [ ! -f "$PYTHON_BIN_PATH" ]; then
	echo "Building python-lab"
	cd python-lab
	bash ./build.sh
	bash ./cut.sh
	cd ..
fi

dd if=/dev/zero of=disk.img bs=4M count=40

mkfs.vfat -F 32 disk.img

sudo mkdir -p mnt
sudo mkdir -p mnt/lib
sudo mount disk.img mnt


echo "Copying aarch64 fat32 aarch64/* to disk"
sudo cp -r ./testcases/aarch64/* ./mnt/
sudo cp -rL ./python-lab/opt/ ./mnt/opt
sudo cp -rL ./python-lab/opt/python3.11/lib/ ./mnt/lib
sudo umount mnt
sudo chmod 777 disk.img
sudo rm -rf mnt
