#!/bin/bash

assert() {
    # expected="$1"
    # input="$2"

    riscv64-unknown-linux-gnu-gcc -static tmp.s -o tmp
    qemu-riscv64 -L $RISCV/sysroot tmp

    echo $?

    # actual="$?"

    # if [ "$actual" -eq "$expected" ]; then
    #     echo "$input => $actual"
    # else
    #     echo "$input => $expected expected, but got $actual"
    #     exit 1
    # fi
}

# assert 0 0
# assert 42 42

assert
