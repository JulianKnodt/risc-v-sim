use crate::mem;
use crate::reg::{Register, RegData};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Exceptions {
  Mem,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Status {
  Running,
  Done,
  Exception(Exceptions),
}

#[derive(PartialEq, Debug)]
pub struct ProgramState<T : RegData> {
  pub regs: Register<T>,
  pub mem: mem::Memory,
  pub status: Status,
}


impl <T : RegData> ProgramState<T> {
  // Sign Extend
  pub fn sx(&self, reg: T) -> T::Signed { reg.to_signed() }
  // Zero Extend
  pub fn zx(&self, reg: T) -> T { reg }
}
pub const HALT: u32 = 0xfeedfeed;

