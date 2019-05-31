li t1, 1
li t2, 2
li t3, 3

sw t1, 200
sw t2, 204
sw t3, 208

sh t1, 212
sh t2, 214
sh t3, 216

sb t1, 218
sb t2, 219
sb t3, 220

lbu t1, 218
lbu t2, 219
lbu t3, 220

lhu t4, 212
lhu t5, 214
lhu t6, 216

lw t7, 200
lw t8, 204
lw t9, 208

.word 0xfeedfeed
