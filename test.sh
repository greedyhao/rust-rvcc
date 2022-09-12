#!/bin/bash

riscv64-unknown-linux-gnu-gcc -static tmp.s -o tmp
qemu-riscv64 -L $RISCV/sysroot tmp

echo $?
