use crate::mem;

const NUM_REGS: usize = 32;
struct ProgramState {
  pc: usize,
  regs: [u32; NUM_REGS],
}

pub fn execute(m: mem::Memory) -> Result<(), ()> {
  let state = ProgramState{pc: 0, regs: Default::default()};
  unimplemented!();
}
