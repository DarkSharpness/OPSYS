.equ TRAP_CONTEXT_ADDRESS, -4096 * 2

    .section .text.trap
    .globl user_handle
    # Align to a page
    .align 12
user_handle:
    # Change to trap frame
    csrw sscratch, sp
    li sp, TRAP_CONTEXT_ADDRESS

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
    # Complete saving all registers

    csrr t0, sscratch
    csrr t1, sepc

    sd t0, 240(sp)  # Uses's stack pointer
    sd t1, 248(sp)  # User's program counter

    ld tp, 256(sp)  # Thread number
    ld t2, 264(sp)  # Kernel satp value
    ld t3, 272(sp)  # Kernel trap handler
    ld sp, 280(sp)  # Kernel stack pointer

    sfence.vma zero, zero
    csrw satp, t2
    sfence.vma zero, zero

    jr t3

    .globl user_handle_end
    .align 3
user_handle_end:

    .globl user_return
    .align 3
user_return:
    # Switch to user's page-table first.
    sfence.vma zero, zero
    csrw satp, a0
    sfence.vma zero, zero

    li sp, TRAP_CONTEXT_ADDRESS

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

    ld t6, 248(sp)  # User's program counter
    csrw sepc, t6   # Restore user's program counter

    ld s10,192(sp)
    ld s11,200(sp)
    ld t3, 208(sp)
    ld t4, 216(sp)
    ld t5, 224(sp)
    ld t6, 232(sp)
    ld sp, 240(sp)  # User's stack pointer

    # Return to user's program
    sret

    .globl user_return_end
    .align 3
user_return_end:

    .globl core_handle
    .align 3
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

    .globl time_handle
    .align 3
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

    # Do not copy xv6's code :)
    # - li a0, 2 
    # - csrw sip, a0
    # That will involve one more instruction.

    # This arranges for a supervisor-level interrupt,
    # after this handle returns.
    csrsi sip, 2

    ld a1, 0(a0)    # 8-byte reload
    ld a2, 8(a0)    # 8-byte reload
    ld a3, 16(a0)   # 8-byte reload
    csrrw a0, mscratch, a0  # Swap back

    mret

    .globl switch_context
    .align 3
switch_context:
    # Store old stack pointer in a0
    # Store new stack pointer in a1

    sd ra, -8(sp)
    sd s0, -16(sp)
    sd s1, -24(sp)
    sd s2, -32(sp)
    sd s3, -40(sp)
    sd s4, -48(sp)
    sd s5, -56(sp)
    sd s6, -64(sp)
    sd s7, -72(sp)
    sd s8, -80(sp)
    sd s9, -88(sp)
    sd s10, -96(sp)
    sd s11, -104(sp)

    sd sp, 0(a0)
    ld sp, 0(a1)

    ld ra, -8(sp)
    ld s0, -16(sp)
    ld s1, -24(sp)
    ld s2, -32(sp)
    ld s3, -40(sp)
    ld s4, -48(sp)
    ld s5, -56(sp)
    ld s6, -64(sp)
    ld s7, -72(sp)
    ld s8, -80(sp)
    ld s9, -88(sp)
    ld s10, -96(sp)
    ld s11, -104(sp)

    ret
switch_context_end:
