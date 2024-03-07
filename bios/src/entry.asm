    .section .text.bios
    .globl bios_start
bios_start:
    la sp, bios_stack_top
    j bios_main

    .section .bss.bios

    .globl bios_stack_low
bios_stack_low:
    .space 4096 * 16 # 64KB

    .globl bios_stack_top
bios_stack_top:
    .globl call_stack_low
call_stack_low:
    .space 4096 * 16

    .globl call_stack_top
call_stack_top:

