    .section .text.drop
    .globl drop_down
drop_down:
    csrw mepc, ra
    mret
drop_down.end:

    .globl time_handle
time_handle:
    mret
time_handle.end:
