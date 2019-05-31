# loops until overflow
main: li t0, 0
addiu t0, 32768
loop:
addiu t0, 32768
bne t0, zero, loop
.align 4
.word 0xfeedfeed
