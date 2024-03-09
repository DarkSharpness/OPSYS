    .section .text.entry
    .globl os_start
os_start:
    la sp, boot_stack_top
    j os_main

    .section .bss.stack
    .globl boot_stack_low
boot_stack_low:
    .space 4096 * 16

    .globl boot_stack_top
boot_stack_top:
