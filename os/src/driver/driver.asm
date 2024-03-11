    .section .text.drop
    .globl drop_mode
drop_mode:
    csrw mepc, ra
    mret
drop_down.end:
