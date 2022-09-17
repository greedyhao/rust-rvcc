#!/bin/bash

file="$1"
file_suffix="$file.s"

riscv64-unknown-linux-gnu-gcc -static "$file_suffix" -o "$file"
qemu-riscv64 -L "$RISCV"/sysroot "$file"

echo $?
