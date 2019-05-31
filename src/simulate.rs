use crate::mem;
use crate::instr::{self, InstrType};

#[derive(Copy, Clone, PartialEq, Debug)]
enum Exceptions {
  Mem,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Status {
  Running,
  Done,
  Exception(Exceptions),
}

const NUM_REGS: usize = 32;
const FP_NUM_REGS: usize = 32;
#[derive(PartialEq, Debug)]
struct ProgramState {
  regs: [u32; NUM_REGS],
  fp_regs: [u32; FP_NUM_REGS],
  pc: u32,
  mem: mem::Memory,
  status: Status,
}


impl ProgramState {
  pub(crate) fn s32(v: u32) -> i32 {
    use std::mem::transmute;
    unsafe { transmute::<u32, i32>(v) }
  }
  #[allow(dead_code)]
  pub(crate) fn s64(v: u32) -> i64 { ProgramState::s32(v) as i64 }
  pub fn sx(&self, v: u32) -> i32 { ProgramState::s32(v) }

  pub(crate) fn z32(v: u32) -> u32 { v }
  #[allow(dead_code)]
  pub(crate) fn z64(v: u32) -> u64 { v as u64 }
  pub fn zx(&self, v: u32) -> u32 { ProgramState::z32(v) }

  pub fn ret(&self, v: i32) -> u32 {
    use std::mem::transmute;
    unsafe { transmute::<i32, u32>(v) }
  }
}

#[cfg(test)]
mod conv_tests {
  use crate::simulate::ProgramState;
  #[test]
  fn test_conversions() {
    let x: u32 = 0x12345678;
    assert_eq!(x.to_ne_bytes(), ProgramState::s32(x).to_ne_bytes());
    assert_eq!(ProgramState::z64(x), 0x00000000_12345678);
  }
}

pub(crate) const HALT: u32 = 0xfeedfeed;

pub fn execute(m: mem::Memory) -> Result<(), ()> {
  let mut ps = ProgramState {
    pc: 0,
    regs: Default::default(),
    fp_regs: Default::default(),
    mem: m,
    status: Status::Running
  };
  while ps.status == Status::Running {
    ps = run_instr(ps);
  }
  ps.regs.iter()
    .enumerate()
    .filter(|(_, v)| **v != 0)
    .for_each(|(i, v)| {
      println!("x{}: {:x}", i, v)
    });
  Ok(())
}

pub fn combine(v: u32, imm: i32) -> u32 {
  if imm < 0 { v - (imm.abs() as u32) }
  else { v + (imm as u32) }
}

fn run_instr(mut ps: ProgramState) -> ProgramState {
  let raw = ps.mem.read(ps.pc as usize, mem::Size::WORD)
    .unwrap_or_else(|_| panic!("Failed to read instr at {}", ps.pc));
  if raw == HALT {
    ps.status = Status::Done;
    return ps
  }
  let instr = instr::decode(raw);
  println!("{:?}", instr);
  match instr {
    instr::InstrType::R(r) => {
      use crate::instr::r::*;
      use crate::instr::RInstr;
      let (rs2, rs1, rd) = (rs2(raw) as usize, rs1(raw) as usize, rd(raw) as usize);
      ps.regs[rd] = match r {
        RInstr::ADD => ps.ret(ps.sx(ps.regs[rs1]) + ps.sx(ps.regs[rs1])),
        RInstr::SUB => ps.ret(ps.sx(ps.regs[rs1]) - ps.sx(ps.regs[rs2])),
        RInstr::SLL => ps.zx(ps.regs[rs1]) << ps.regs[rs2],
        RInstr::SLT => if ps.sx(ps.regs[rs1]) < ps.sx(ps.regs[rs2]) { 1 } else { 0 },
        RInstr::SLTU => if ps.zx(ps.regs[rs1]) < ps.zx(ps.regs[rs2]) { 1 } else { 0 },
        RInstr::XOR => ps.zx(ps.regs[rs1]) ^ ps.zx(ps.regs[rs2]),
        RInstr::SRL => ps.zx(ps.regs[rs1]) >> ps.regs[rs2],
        RInstr::SRA => ps.ret(ps.sx(ps.regs[rs1]) >> ps.regs[rs2]),
        RInstr::OR => ps.zx(ps.regs[rs1]) | ps.zx(ps.regs[rs2]),
        RInstr::AND => ps.zx(ps.regs[rs1]) & ps.zx(ps.regs[rs2]),
        RInstr::SLLI => ps.zx(ps.regs[rs1]) << rs2,
        RInstr::SRLI => ps.zx(ps.regs[rs1]) >> rs2,
        RInstr::SRAI => ps.ret(ps.sx(ps.regs[rs1]) >> rs2),
      };
    },
    InstrType::I(i) => {
      use crate::instr::i::*;
      use crate::instr::IInstr;
      let (rs1, rd) = (rs1(raw) as usize, rd(raw) as usize);
      let (sx_imm, zx_imm) = (sx_imm(raw), zx_imm(raw));
      ps.regs[rd] = match i {
        IInstr::ADDI => ps.ret(ps.sx(ps.regs[rs1]) + sx_imm),
        IInstr::SLTI => if ps.sx(ps.regs[rs1]) < sx_imm { 1 } else { 0 },
        IInstr::SLTIU => if ps.zx(ps.regs[rs1]) < zx_imm { 1 } else { 0 },
        IInstr::XORI => ps.zx(ps.regs[rs1]) ^ zx_imm,
        IInstr::ORI => ps.zx(ps.regs[rs1]) | zx_imm,
        IInstr::ANDI => ps.zx(ps.regs[rs1]) & zx_imm,
        IInstr::JALR => {
          let result = ps.pc;
          ps.pc = ps.ret((ps.sx(ps.regs[rs1]) + sx_imm) & -2);
          result
        },
        IInstr::LW =>
          ps.mem.read((ps.sx(ps.regs[rs1])+sx_imm) as usize, mem::Size::WORD)
            .unwrap_or_else(|_| ps.regs[rd]),
        IInstr::LH =>
          ps.mem.read((ps.sx(ps.regs[rs1])+sx_imm) as usize, mem::Size::HALF)
            .unwrap_or_else(|_| ps.regs[rd]),
        IInstr::LB =>
          ps.mem.read((ps.sx(ps.regs[rs1])+sx_imm) as usize, mem::Size::BYTE)
            .unwrap_or_else(|_| ps.regs[rd]),
        IInstr::LHU =>
          ps.mem.read_signed((ps.sx(ps.regs[rs1])+sx_imm) as usize, mem::Size::HALF)
            .unwrap_or_else(|_| ps.regs[rd]),
        IInstr::LBU =>
          ps.mem.read_signed((ps.sx(ps.regs[rs1])+sx_imm) as usize, mem::Size::BYTE)
            .unwrap_or_else(|_| ps.regs[rd]),
      };
    },
    InstrType::S(s) => {
      use crate::instr::s::*;
      use crate::instr::SInstr;
      let (rs1, rs2, imm) = (rs1(raw) as usize, rs2(raw) as usize, imm(raw));
      let size = match s {
        SInstr::SB => mem::Size::BYTE,
        SInstr::SH => mem::Size::HALF,
        SInstr::SW => mem::Size::WORD,
      };
      if let Err(e) = ps.mem.write((ps.regs[rs1] + imm) as usize, ps.regs[rs2], size) {
          println!("{:?}", e);
          ps.status = Status::Exception(Exceptions::Mem);
      };
    },
    InstrType::B(b) => {
      use crate::instr::b::*;
      use crate::instr::BInstr;
      let (rs2, rs1) = (rs2(raw) as usize, rs1(raw) as usize);
      let branch = match b {
        BInstr::BEQ => ps.regs[rs1] == ps.regs[rs2],
        BInstr::BNE => ps.regs[rs1] != ps.regs[rs2],
        BInstr::BLT => ps.sx(ps.regs[rs1]) < ps.sx(ps.regs[rs2]),
        BInstr::BGE => ps.sx(ps.regs[rs1]) >= ps.sx(ps.regs[rs2]),
        BInstr::BLTU => ps.zx(ps.regs[rs1]) < ps.zx(ps.regs[rs2]),
        BInstr::BGEU => ps.zx(ps.regs[rs1]) >= ps.zx(ps.regs[rs2]),
      };
      if branch {
        ps.pc = combine(ps.pc, imm(raw)) - (mem::WORD_SIZE as u32);
      };
    },
    InstrType::U(u) => {
      use crate::instr::u::*;
      use crate::instr::UInstr;
      let (rd, imm) = (rd(raw) as usize, imm(raw));
      match u {
        UInstr::LUI => ps.regs[rd] = imm,
        UInstr::AUIPC => ps.regs[rd] = imm + (ps.pc as u32),
      };
    },
    InstrType::J(j) => {
      use crate::instr::j::{rd, offset};
      use crate::instr::JInstr;
      let (rd, offset) = (rd(raw) as usize, offset(raw));
      match j {
        JInstr::JAL => {
          ps.regs[rd] = ps.pc;
          ps.pc = combine(ps.pc, offset) - (mem::WORD_SIZE as u32);
        },
      };
    },
  };
  ps.regs[0] = 0;
  ps.pc += mem::WORD_SIZE as u32;
  return ps;
}

