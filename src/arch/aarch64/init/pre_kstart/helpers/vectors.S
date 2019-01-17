    //  Exception vector stubs
    //
    //  The hex values in x18 are to aid debugging
    //  Unhandled exceptions spin in a wfi loop for the moment
    //  This can be macro-ified

    .align 11
exception_vector_base:

    .align 7
    mov     x18, #0xb0b0
__vec_00:
    wfi
    b __vec_00

    .align 7
    mov     x18, #0xb0b1
__vec_01:
    wfi
    b __vec_01

    .align 7
    mov     x18, #0xb0b2
__vec_02:
    wfi
    b __vec_02

    .align 7
    mov     x18, #0xb0b3
__vec_03:
    wfi
    b __vec_03

    .align 7
    mov     x18, #0xb0b4
__vec_04:
    wfi
    b __vec_04

    .align 7
    b       do_irq                              // First level interrupt handler

    .align 7
    mov     x18, #0xb0b6
__vec_06:
    wfi
    b __vec_06

    .align 7
    mov     x18, #0xb0b7
__vec_07:
    wfi
    b __vec_07

    .align 7
    b       do_syscall                          // Syscall handler

    .align 7
    mov     x18, #0xb0ba
__vec_010:
    wfi
    b __vec_010

    .align 7
    mov     x18, #0xb0bb
__vec_011:
    wfi
    b __vec_011

    .align 7
    mov     x18, #0xb0bc
__vec_012:
    wfi
    b __vec_012

    .align 7
    mov     x18, #0xb0bd
__vec_013:
    wfi
    b __vec_013

    .align 7
    mov     x18, #0xb0be
__vec_014:
    wfi
    b __vec_014

    .align 7
    mov     x18, #0xb0bf
__vec_015:
    wfi
    b __vec_015

    .align 7
exception_vector_end:
