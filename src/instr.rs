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
  BEQ, BNE, BLT, BGE, BLTU, BGEU, SB, SH, SW, LUI, AUIPC,
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

fn opcode(v: u32) -> u32 { v & 0b111111 }
// R-type functions
pub(crate) mod r {
  pub fn funct7(v: u32) -> u32 { v >> 25 }
  pub fn rs2(v: u32) -> u32 { (v >> 18) & 0b11111 }
  pub fn rs1(v: u32) -> u32 { (v >> 13) & 0b11111 }
  pub fn funct3(v: u32) -> u32 { (v >> 8) & 0b111 }
  pub fn rd(v: u32) -> u32 { v & 0b11111 }
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
  // TODO implement joining immediates
  pub fn imm_upper(v: u32) -> u32 { v >> 25 }
  pub use crate::instr::r::{rs2,rs1,funct3};
  use crate::instr::r::{rd};
  pub fn imm_lower(v: u32) -> u32 { rd(v) }
}

pub(crate) mod u {
  pub fn imm(v: u32) -> u32 { v >> 12 }
  pub use crate::instr::r::{rd};
}
