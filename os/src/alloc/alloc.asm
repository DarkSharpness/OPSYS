    .section .data.pagetable
    .globl kernel_pagetable
    .p2align 13
kernel_pagetable:
    .space 4096 * 1

    .section .text.pagetable
    .globl get_pagetable
get_pagetable:
    la a0, kernel_pagetable
    ret
get_pagetable_end:
