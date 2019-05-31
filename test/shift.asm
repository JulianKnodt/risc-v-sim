main:
li x3, 0x11111111
sll x3, x3, 16
# x3 = 0x11110000

li x4, 0x11111111
srl x4, x4, 16
# x4 = 0x00001111
done:
.word 0xfeedfeed
