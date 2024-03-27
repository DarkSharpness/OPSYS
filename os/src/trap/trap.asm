    .section .text.trap
    .globl user_handle
    .align 8
user_handle:
/*
    This trampoline should be placed at 0x80001000.
    This page has the same physical address as virtual one.

    Trap frame layout (at user-space virtual 0x80000000):
        [0x000, 0x0f8): 31 General purpose registers.
        [0x0f8, 0x100): User's program counter.
        [0x100, 0x108): Thread number of the process.
        [0x108, 0x110): Kernel stack pointer.

    This trampoline page (0x80001000) should be executable-only.
    That trap frame page (0x80000000) should be read/write-only.
    Both page can only be accessed in supervisor mode, which
    means the U bit of those 2 pages should be 0.
*/

    # Change to trap frame
    csrw sscratch, sp
    li sp, 0x80000000

    # Save all registers on user's trap frame
    sd ra, 0(sp)
    sd gp, 8(sp)
    sd tp, 16(sp)
    sd t0, 24(sp)
    sd t1, 32(sp)
    sd t2, 40(sp)
    sd s0, 48(sp)
    sd s1, 56(sp)

    sd a0, 64(sp)
    sd a1, 72(sp)
    sd a2, 80(sp)
    sd a3, 88(sp)
    sd a4, 96(sp)
    sd a5, 104(sp)
    sd a6, 112(sp)
    sd a7, 120(sp)

    sd s2, 128(sp)
    sd s3, 136(sp)
    sd s4, 144(sp)
    sd s5, 152(sp)
    sd s6, 160(sp)
    sd s7, 168(sp)
    sd s8, 176(sp)
    sd s9, 184(sp)

    sd s10,192(sp)
    sd s11,200(sp)
    sd t3, 208(sp)
    sd t4, 216(sp)
    sd t5, 224(sp)
    sd t6, 232(sp)

    csrr t0, sscratch
    csrr t1, sepc

    sd t0, 240(sp)  # Uses's stack pointer
    sd t1, 248(sp)  # User's program counter

    ld tp, 256(sp)  # Thread number
    ld sp, 264(sp)  # Kernel stack pointer

    # Wait old memory operation to finish
    # Then, disable page table. Our kernel is not using it.
    sfence.vma zero, zero
    csrw satp, zero
    sfence.vma zero, zero

    # Jump to the real user trap handler
    la t0, user_trap
    jr t0

user_handle.end:

    .globl user_handle_end
    .align 8
user_handle_end:

    .globl user_return
    .align 8
user_return:
/*
    This function should be placed at 0x80001800.
    It shares the same page with trampoline page.
*/

    # Switch to user's page-table first.
    sfence.vma zero, zero
    csrw satp, a0
    sfence.vma zero, zero

    li sp, 0x80000000

    # Restore all registers from user's trap frame
    ld ra, 0(sp)
    ld gp, 8(sp)
    ld tp, 16(sp)
    ld t0, 24(sp)
    ld t1, 32(sp)
    ld t2, 40(sp)
    ld s0, 48(sp)
    ld s1, 56(sp)

    ld a0, 64(sp)
    ld a1, 72(sp)
    ld a2, 80(sp)
    ld a3, 88(sp)
    ld a4, 96(sp)
    ld a5, 104(sp)
    ld a6, 112(sp)
    ld a7, 120(sp)

    ld s2, 128(sp)
    ld s3, 136(sp)
    ld s4, 144(sp)
    ld s5, 152(sp)
    ld s6, 160(sp)
    ld s7, 168(sp)
    ld s8, 176(sp)
    ld s9, 184(sp)

    ld s10,192(sp)
    ld s11,200(sp)
    ld t3, 208(sp)
    ld t4, 216(sp)
    ld t5, 224(sp)

    ld t6, 248(sp)  # User's program counter
    csrw sepc, t6

    ld t6, 232(sp)
    ld sp, 240(sp)  # User's stack pointer

    # Return to user's program
    sret
user_return.end:

    .globl user_return_end
    .align 8
user_return_end:


    .globl core_handle
    .align 8
core_handle:
    addi sp, sp, -192

    sd ra, 0(sp)
    sd gp, 8(sp)
    sd tp, 16(sp)
    sd t0, 24(sp)
    sd t1, 32(sp)
    sd t2, 40(sp)
    sd a0, 48(sp)
    sd a1, 56(sp)

    sd a2, 64(sp)
    sd a3, 72(sp)
    sd a4, 80(sp)
    sd a5, 88(sp)
    sd a6, 96(sp)
    sd a7, 104(sp)
    sd t3, 112(sp)
    sd t4, 120(sp)

    sd t5, 128(sp)
    sd t6, 136(sp)

    # Jump to the real handler
    # This will save those saved poiner for us.
    call core_trap

    ld ra, 0(sp)
    ld gp, 8(sp)
    ld tp, 16(sp)
    ld t0, 24(sp)
    ld t1, 32(sp)
    ld t2, 40(sp)
    ld a0, 48(sp)
    ld a1, 56(sp)

    ld a2, 64(sp)
    ld a3, 72(sp)
    ld a4, 80(sp)
    ld a5, 88(sp)
    ld a6, 96(sp)
    ld a7, 104(sp)
    ld t3, 112(sp)
    ld t4, 120(sp)

    ld t5, 128(sp)
    ld t6, 136(sp)

    addi sp, sp, 192
    sret
core_handle.end:

    .globl time_handle
    .align 8
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

    .globl return_to_user
    .align 8
return_to_user:
    li t0, 0x80001800   # call user_return
    jr t0
return_to_user.end:
