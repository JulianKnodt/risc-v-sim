# RISCV simulator

A simulator for RISCV written in Rust.
I'm starting with the basic RV32I instructions and moving forward from there adding more
optional variants. We'll see how far I build it up.

## Structure

The project compiles a single binary, which can be used with the following flags:
```
-io | --inorder # for running pipelined execution in order
-ooo | --outoforder # for running pipelined execution out of order
# By default it runs a simulator without any form of pipelining, just simulating the
# instructions themselves
-v | --verbose # print out register file after execution
-m | --mem <usize> # size of memory in bytes
# additional arguments treated as riscv binaries
```

## Implementation Notes:

Does not implement stalls on RAW, or Load Use, but it would not be hard to implement.
Currently out of order fails when there are instructions with dependencies, because I'm
investigating how to accurately simulate passing back results that have not been retired. It
would be simplest to keep two copies of a register file, and only update the visible one, but
that seems inaccurate, because I do not think that it would necessarily reflect how a computer
might store it.
