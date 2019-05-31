main:
addi t0, t0, 1
addi t1, t1, 1024
sb t0, 0(t1)
lbu t0, 0(t1)
andi t0, t0, 0
addi t0, t0, 0x4321
sb t0, 0(t1)
sh t0, 2(t1)
sw t0, 4(t1)
lbu s0, 0(t1)
lbu s1, 1(t1)
lbu s2, 2(t1)
lbu s3, 3(t1)
lhu s4, 4(t1)
lhu s5, 6(t1)
addi t2, 0x1111
lui t2, 0x9876
lui t3, 0x0321
addi t2, 0x1234
sh t2, 8(t1)
lhu t4, 8(t1)
lhu t5, 10(t1)
lw t6, 8(t1)
done:
.word 0xfeedfeed
