main:
li x1, 0xFFFFFFFF
li x2, 0xFFFFFFFE
li x3, 0x00000001
li x4, 0xEFFFFFFF

slt x8, x1, x2           #   0
sltu x9, x1, x2          #   0
slti x10, x2, 0   		#   1
sltiu x11, x2, 0   	#   0

slt x12, x2, x4           #   0
sltu x13, x2, x4          #   0
sltu x14, x4, x2          #   1
slti x15, x1, 1		#   1
sltiu x16, x1, 1  		#   0
done:
.word 0xfeedfeed
