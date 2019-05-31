# Compute first twelve Fibonacci numbers and put in array
main: la   x1, 68          # load address of array
la   x6, 116         # load address of size variable
lw   x6, 0(x6)      # load array size
li   x3, 1           # 1 is first and second Fib. number
sw   x3, 0(x1)      # F[0] = 1
sw   x3, 4(x1)      # F[1] = F[0] = 1
addi x2, x6, -2     # Counter for loop, will execute (size-2) times
loop: lw   x4, 0(x1)      # Get value from array F[n]
lw   x5, 4(x1)      # Get value from array F[n+1]
add  x3, x4, x5    # x3 = F[n] + F[n+1]
sw   x3, 8(x1)      # Store F[n+2] = F[n] + F[n+1] in array
addi x1, x1, 4      # increment address of Fib. number source
addi x2, x2, -1     # decrement loop counter
bne  x2, zero, loop # repeat if not finished yet.
.word 0xfeedfeed
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0x0             # size of "array"
.word 0xc             # size of "array"
