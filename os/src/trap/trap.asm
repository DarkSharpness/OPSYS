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

    .globl time_handle
    .align 4
time_handle:
    # Why I choose a0 ~ a3 ?
    # Because they can help generate compressed instruction.
    # See c.sd / c.ld in RISC-V spec.

    csrrw a0, mscratch, a0  # Swap with mscratch
    sd a1, 0(a0)    # 8-byte spill
    sd a2, 8(a0)    # 8-byte spill
    sd a3, 16(a0)   # 8-byte spill

    ld a1, 24(a0)   # MTIMECMP address
    ld a2, 32(a0)   # Time interval

    ld a3, 0(a1)    # MTIMECMP value
    add a3, a3, a2  # New MTIMECMP value
    sd a3, 0(a1)    # Update new MTIMECMP

    # Do not copy xv6's code.
    # - li a0, 2 
    # - csrw sip, a0
    # That will involve one more instruction.

    # This arranges for a supervisor-level interrupt,
    # after this handle returns.
    csrwi sip, 2

    ld a1, 0(a0)    # 8-byte reload
    ld a2, 8(a0)    # 8-byte reload
    ld a3, 16(a0)   # 8-byte reload
    csrrw a0, mscratch, a0  # Swap back

    mret
time_handle.end:
