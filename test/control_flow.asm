# test out various control flows, registers should come out in order
main:
li t0, 0
addi t7, 1
move t1, t7
li s1, 1

beg:
beq t0, zero, hit
addi t7, 1
move t3, t7
li s3, 3
jal fn
addi t7, 1
move t5, t7
li s5, 5
j done

hit:
li t0, -1
addi t7, 1
move t2, t7
li s2, 2
j beg

fn:
addi t7, 1
move t4, t7
li s4, 4
jr ra

done:
addi t7, 1
move t6, t7
addi t7, 1
li s6, 6
.align 4
.word 0xfeedfeed
