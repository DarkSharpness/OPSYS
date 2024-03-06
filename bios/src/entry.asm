    .section .text.bios
    .globl bios_start
bios_start:
    li t0, 0x80200000
    jr t0
