main:
# an sc instruction before ll always fails
li t0, 42       # set t0 to 24
li s0, 1024     # s0 is the start of memory
sc t0, 0(s0)   # try sc before any ll, t0 should be 0
lw t0, 0(s0)   # make sure memory isn't altered

# an sc instruction clears any reservation taken out by ll
li t0, 42       # to = 42
sw t0, 4(s0)
ll t0, 4(s0)   # t0 = 42
li t0, 69       # t0 = 69
sc t0, 4(s0)   # t0 = 1
lw t1, 4(s0)   # t1 = 69
li t0, 42       # t0 = 42
sc t0, 4(s0)   # t0 = 0
lw t2, 4(s0)   # t2 = 69

# an sc to a different resAddress always fails
sw t1, 8(s0)
ll t1, 8(s0)      # t1 = 69
sc t1, 12(s0)     # t1 = 0
lw t3, 12(s0)     # t3 = 0

# at most one reservation at a time
li t0, 13
sw t0, 0(s0)
ll t4, 0(s0)      # t4 = 13
ll t5, 4(s0)      # t5 = 69
sc t5, 0(s0)      # t5 = 0
sc t4, 4(s0)      # t4 = 0

# a write that overlaps with resAddress voids it
ll t2, 8(s0)      # t2 = 69
li t6, 1
sh t6, 8(s0)
sc t2, 8(s0)      # t2 = 0

ll t6, 8(s0)      # t6 = 0x000100045
sb t2, 11(s0)
sc t6, 8(s0)      # t6 = 0
done:
.word 0xfeedfeed
