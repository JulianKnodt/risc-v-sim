# Testing various add functions
li x3, -3
add x20, zero, x3
li x6, 6
add x1, x5, x6
addi x2, x1, -3

lui x7, 10
add x7, x7, x7
.word 0xfeedfeed
