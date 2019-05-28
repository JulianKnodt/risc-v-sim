#![allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum RInstr {
  SLLI, SRLI, SRAI, ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IInstr {
  JALR, LB, LH, LW, LBU, LHU, ADDI, SLTI, SLTIU, XORI, ORI, ANDI,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BInstr {
  BEQ, BNE, BLT, BGE, BLTU, BGEU,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum JInstr {
  JAL,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Instr {
  FENCE, FENCEI, ECALL, EBREAK,
  CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SInstr {
  SB, SH, SW,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum UInstr {
  LUI, AUIPC,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum InstrType {
  R(RInstr), I(IInstr), S(SInstr), B(BInstr), U(UInstr), J(JInstr),
}

pub(crate) fn decode(instr: u32) -> InstrType {
  match opcode(instr) {
    0b1101111 => InstrType::J(JInstr::JAL),
    0b0110111 => InstrType::U(UInstr::LUI),
    0b0010111 => InstrType::U(UInstr::AUIPC),
    0b1100111 if i::funct3(instr) == 0 => InstrType::I(IInstr::JALR),
    0b1100011 => match s::funct3(instr) {
      0b000 => InstrType::B(BInstr::BEQ),
      0b001 => InstrType::B(BInstr::BNE),
      0b100 => InstrType::B(BInstr::BLT),
      0b101 => InstrType::B(BInstr::BGE),
      0b110 => InstrType::B(BInstr::BLTU),
      0b111 => InstrType::B(BInstr::BGEU),
      funct3 => panic!("Unexpected funct3 for opcode 0b1100011 : {}", funct3),
    },
    0b0000011 => match i::funct3(instr) {
      0b000 => InstrType::I(IInstr::LB),
      0b001 => InstrType::I(IInstr::LH),
      0b010 => InstrType::I(IInstr::LW),
      0b100 => InstrType::I(IInstr::LBU),
      0b101 => InstrType::I(IInstr::LHU),
      funct3 => panic!("Unexpected funct3 for opcode 0b0000011: {}", funct3),
    },
    0b0100011 => match s::funct3(instr) {
      0b000 => InstrType::S(SInstr::SB),
      0b001 => InstrType::S(SInstr::SH),
      0b010 => InstrType::S(SInstr::SW),
      funct3 => panic!("Unexpected funct3 for opcode 0b0100011: {}", funct3),
    },
    0b0010011 => match i::funct3(instr) {
      0b000 => InstrType::I(IInstr::ADDI),
      0b010 => InstrType::I(IInstr::SLTI),
      0b011 => InstrType::I(IInstr::SLTIU),
      0b100 => InstrType::I(IInstr::XORI),
      0b110 => InstrType::I(IInstr::ORI),
      0b111 => InstrType::I(IInstr::ANDI),
      0b001 => InstrType::R(RInstr::SLLI),
      0b101 if r::funct7(instr) == 0 => InstrType::R(RInstr::SRLI),
      0b101 => InstrType::R(RInstr::SRAI),
      funct3 => panic!("Unexpected funct3 for opcode 0b0010111: {}", funct3),
    },
    0b0110011 => match (r::funct7(instr), r::funct3(instr)) {
      (0, 0b000) => InstrType::R(RInstr::ADD),
      (32, 0b000)=> InstrType::R(RInstr::SUB),
      (0, 0b001) => InstrType::R(RInstr::SLL),
      (0, 0b010) => InstrType::R(RInstr::SLT),
      (0, 0b011) => InstrType::R(RInstr::SLTU),
      (0, 0b100) => InstrType::R(RInstr::XOR),
      (0, 0b101) => InstrType::R(RInstr::SRL),
      (32, 0b101) => InstrType::R(RInstr::SRA),
      (0, 0b110) => InstrType::R(RInstr::OR),
      (0, 0b111) => InstrType::R(RInstr::AND),
      (f7, f3) => panic!("Unexpected funct7 & funct3 for opcode 0b0110011: {}, {}", f7, f3),
    },
    _ => unimplemented!(),
  }
}


const OPCODE_MASK: u32 = 0b1111111;
fn opcode(v: u32) -> u32 { v & OPCODE_MASK }

// R-type functions
pub(crate) mod r {
  const REG_MASK: u32 = 0b11111;
  const OPCODE_SIZE: u32 = 7;
  pub fn funct7(v: u32) -> u32 { v >> 25 }
  pub fn rs2(v: u32) -> u32 { (v >> 20) & REG_MASK }
  pub fn rs1(v: u32) -> u32 { (v >> 15) & REG_MASK }
  pub fn funct3(v: u32) -> u32 { (v >> 12) & 0b111 }
  pub fn rd(v: u32) -> u32 { (v >> OPCODE_SIZE) & REG_MASK }
}

pub(crate) mod i {
  pub fn sx_imm(v: u32) -> i32 {
    use std::mem::transmute;
    let v = unsafe { transmute::<u32, i32>(v) };
    v >> 20
  }
  pub fn zx_imm(v: u32) -> u32 { v >> 20 }
  pub use crate::instr::r::{rs1, funct3, rd};
}

pub(crate) mod s {
  pub use crate::instr::r::{rs2,rs1,funct3};
  use crate::instr::r::{rd};
  pub fn imm(v: u32) -> u32 {
    ((v >> 25) << 5) | rd(v)
  }
}

pub(crate) mod b {
  pub use crate::instr::s::{rs2, rs1, funct3};
  use crate::instr::s::imm as s_imm;
  pub fn imm(v: u32) -> u32 {
    let s = s_imm(v);
    ((s >> 11) << 12) |
      ((s & 0b1) << 11) |
      (((v >> 25) & 0b111111) << 5) |
      (((v >> 8) & 0b11111) << 1)
  }

  #[test]
  fn b_imm_test() {
    let v: u32 = 0b1111111_00000_00000_000_11111_0000000;
    assert_eq!(imm(v), 0b1111111111110);

    let v: u32 = 0b0111111_00000_00000_000_11111_0000000;
    assert_eq!(imm(v), 0b0111111111110);

    let v: u32 = 0b1111111_00000_00000_000_11110_0000000;
    assert_eq!(imm(v), 0b1011111111110);

    let v: u32 = 0b1000000_00000_00000_000_11111_0000000;
    assert_eq!(imm(v), 0b1100000011110);

    let v: u32 = 0b1111111_00000_00000_000_00001_0000000;
    assert_eq!(imm(v), 0b1111111100000);
  }
}

pub(crate) mod u {
  pub fn imm(v: u32) -> u32 { v >> 12 }
  pub use crate::instr::r::{rd};
}

pub(crate) mod j {
  pub use crate::instr::r::rd;
  pub fn offset(v: u32) -> u32 {
    unimplemented!();
  }
}







