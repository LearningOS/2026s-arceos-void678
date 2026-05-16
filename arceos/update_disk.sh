#!/bin/sh

if [ $# -ne 1 ]; then
    printf "Usage: ./update.sh [userapp path]\n"
    exit
fi

FILE=$1

if [ ! -f $FILE ]; then
    printf "File '$FILE' doesn't exist!\n"
    exit
fi

if [ ! -f ./disk.img ]; then
    printf "disk.img doesn't exist! Please 'make disk_img'\n"
    exit
fi

printf "Write file '$FILE' into disk.img\n"

mkdir -p ./mnt
if command -v mcopy >/dev/null 2>&1; then
    mmd -i ./disk.img ::/sbin 2>/dev/null || true
    mcopy -o -i ./disk.img "$FILE" "::/sbin/$(basename "$FILE")"
else
    sudo mount ./disk.img ./mnt
    sudo mkdir -p ./mnt/sbin
    sudo cp "$FILE" ./mnt/sbin
    sudo umount ./mnt
fi
rm -rf mnt
