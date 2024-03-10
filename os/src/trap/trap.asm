    .section .text.trap
    .globl user_handle
user_handle:
    # Change to trap frame
    csrw sscratch, sp
    li sp, 0x80000000

    # Save all registers on user's stack
    sd ra, 0(sp)
    csrr ra, sscratch
    sd ra, 8(sp)    # Old sp in now in ra
    sd gp, 16(sp)
    sd tp, 24(sp)
    sd t0, 32(sp)
    sd t1, 40(sp)
    sd t2, 48(sp)
    sd s0, 56(sp)

    sd s1, 64(sp)
    sd a0, 72(sp)
    sd a1, 80(sp)
    sd a2, 88(sp)
    sd a3, 96(sp)
    sd a4, 104(sp)
    sd a5, 112(sp)
    sd a6, 120(sp)

    sd a7, 128(sp)
    sd s2, 136(sp)
    sd s3, 144(sp)
    sd s4, 152(sp)
    sd s5, 160(sp)
    sd s6, 168(sp)
    sd s7, 176(sp)
    sd s8, 184(sp)

    sd s9, 192(sp)
    sd s10,200(sp)
    sd s11,208(sp)
    sd t3, 216(sp)
    sd t4, 224(sp)
    sd t5, 232(sp)
    sd t6, 240(sp)

    # Wait old memory operation to finish
    # Then, disable page table. Our kernel is not using it.
    sfence.vma zero, zero
    csrw satp, zero
    sfence.vma zero, zero

    # Jump to the real handler
    j user_trap

user_handle.end:

    .globl user_return
user_return:
    # Wait old memory operation to finish
    # Then, enable page table. User has its own page table.
    sfence.vma zero, zero
    csrw satp, a0
    sfence.vma zero, zero

    li sp, 0x80000000

    # Load all registers from user's stack
    ld s1, 64(sp)
    ld a0, 72(sp)
    ld a1, 80(sp)
    ld a2, 88(sp)
    ld a3, 96(sp)
    ld a4, 104(sp)
    ld a5, 112(sp)
    ld a6, 120(sp)

    ld a7, 128(sp)
    ld s2, 136(sp)
    ld s3, 144(sp)
    ld s4, 152(sp)
    ld s5, 160(sp)
    ld s6, 168(sp)
    ld s7, 176(sp)
    ld s8, 184(sp)

    ld s9, 192(sp)
    ld s10,200(sp)
    ld s11,208(sp)
    ld t3, 216(sp)
    ld t4, 224(sp)
    ld t5, 232(sp)
    ld t6, 240(sp)

    ld gp, 16(sp)
    ld tp, 24(sp)
    ld t0, 32(sp)
    ld t1, 40(sp)
    ld t2, 48(sp)
    ld s0, 56(sp)
    ld ra, 0(sp)
    ld sp, 8(sp)

    # Return to user mode
    sret
user_return.end:
