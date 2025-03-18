.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # 交换寄存器sp和寄存器sscratch的值，使sp指向内核栈顶
    csrrw sp, sscratch, sp
    # 保存x1、x3、x5到x31
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # 保存寄存器sstatus和sepc的值到堆栈
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 保存寄存器sscratch的值到堆栈
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 加载新的satp、sepc、sp
    ld t0, 34*8(sp)
    ld t1, 36*8(sp)
    ld sp, 35*8(sp)
    csrw satp, t0
    sfence.vma
    jr t1

__restore:
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ld sp, 2*8(sp)
    sret
