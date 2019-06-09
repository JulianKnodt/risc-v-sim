main:
lui x1, 0xFFFFF
ori x1, x1, 0xFF
# 0xFFFFF0FF

li x2, 0xFFFFFFFF
andi x2, x2, 0xFF
# 0x000000FF

ori x3, x3, 0xFF
# 0x000000FF

li x4, 0xFFFFFFFF
add x4, x4, 1
# 0

xori x5, x5, 0xFF
# 0xff

li x6, 1
sll x6, x6, 31
# 0x80000000

li x7, -1
sra x7, x7, 31
# 0xFFFFFFFF

li x8, -1
srl x8, x8, 31
# 0x1

done:
.word 0xfeedfeed


