beq zero, zero, done

never:
li x11,1
.word 0xfeedfeed

done:
li x10, -1
.word 0xfeedfeed

