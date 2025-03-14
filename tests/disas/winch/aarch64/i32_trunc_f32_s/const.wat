;;! target = "aarch64"
;;! test = "winch"

(module
    (func (result i32)
        (f32.const 1.0)
        (i32.trunc_f32_s)
    )
)
;; wasm[0]::function[0]:
;;       stp     x29, x30, [sp, #-0x10]!
;;       mov     x29, sp
;;       mov     x28, sp
;;       mov     x9, x0
;;       sub     x28, x28, #0x10
;;       mov     sp, x28
;;       stur    x0, [x28, #8]
;;       stur    x1, [x28]
;;       mov     x16, #0x3f800000
;;       fmov    s0, w16
;;       fcmp    s0, s0
;;       b.vs    #0x64
;;   30: mov     x16, #0xcf000000
;;       fmov    s31, w16
;;       fcmp    s31, s0
;;       b.le    #0x68
;;   40: mov     x16, #0x4f000000
;;       fmov    s31, w16
;;       fcmp    s31, s0
;;       b.ge    #0x6c
;;   50: fcvtzs  w0, s0
;;       add     x28, x28, #0x10
;;       mov     sp, x28
;;       ldp     x29, x30, [sp], #0x10
;;       ret
;;   64: .byte   0x1f, 0xc1, 0x00, 0x00
;;   68: .byte   0x1f, 0xc1, 0x00, 0x00
;;   6c: .byte   0x1f, 0xc1, 0x00, 0x00
