#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::u32;
use riscv::mem;

fn main() {
  env::args().for_each(|s| {
    run(s).expect("Error");
  });
}

enum Instr {
  LUI, AUIPC, JAL, JALR,
  BEQ, BNE, BLT, BGE, BLTU, BGEU,
  LB, LH, LW, LBU, LHU, SB, SH, SW,

  ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI,
  SRAI, ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA,
  OR, AND, FENCE, FENCEI, ECALL, EBREAK,
  CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI,
}

enum InstrType {
  R, I, S, U,
}


fn from_op(instr: u32) -> (Instr, InstrType) {
  match opcode(instr) {
    0b1101111 => (Instr::JAL, InstrType::U),
    0b0110111 => (Instr::LUI, InstrType::U),
    0b0010111 => (Instr::AUIPC, InstrType::U),
    0b1100111 if i::funct3(instr) == 0 => (Instr::JALR, InstrType::I),
    0b1100011 => match s::funct3(instr) {
      0b000 => (Instr::BEQ, InstrType::S),
      0b001 => (Instr::BNE, InstrType::S),
      0b100 => (Instr::BLT, InstrType::S),
      0b101 => (Instr::BGE, InstrType::S),
      0b110 => (Instr::BLTU, InstrType::S),
      0b111 => (Instr::BGEU, InstrType::S),
      funct3 => panic!("Unexpected funct3 for opcode 0b1100011 : {}", funct3),
    },
    0b0000011 => match i::funct3(instr) {
      0b000 => (Instr::LB, InstrType::I),
      0b001 => (Instr::LH, InstrType::I),
      0b010 => (Instr::LW, InstrType::I),
      0b100 => (Instr::LBU, InstrType::I),
      0b101 => (Instr::LHU, InstrType::I),
      funct3 => panic!("Unexpected funct3 for opcode 0b0000011: {}", funct3),
    },
    0b0100011 => match s::funct3(instr) {
      0b000 => (Instr::SB, InstrType::S),
      0b001 => (Instr::SH, InstrType::S),
      0b010 => (Instr::SW, InstrType::S),
      funct3 => panic!("Unexpected funct3 for opcode 0b0100011: {}", funct3),
    },
    0b0010011 => match i::funct3(instr) {
      0b000 => (Instr::ADDI, InstrType::I),
      0b010 => (Instr::SLTI, InstrType::I),
      0b011 => (Instr::SLTIU, InstrType::I),
      0b100 => (Instr::XORI, InstrType::I),
      0b110 => (Instr::ORI, InstrType::I),
      0b111 => (Instr::ANDI, InstrType::I),
      funct3 => panic!("Unexpected funct3 for opcode 0b0010111: {}", funct3),
    },
    0b0110011 => match (r::funct7(instr), r::funct3(instr)) {
      (0, 0b000) => (Instr::ADD, InstrType::R),
      (32, 0b000) => (Instr::SUB, InstrType::R),
      (0, 0b001) => (Instr::SLL, InstrType::R),
      (0, 0b010) => (Instr::SLT, InstrType::R),
      (0, 0b011) => (Instr::SLTU, InstrType::R),
      (0, 0b100) => (Instr::XOR, InstrType::R),
      (0, 0b101) => (Instr::SRL, InstrType::R),
      (32, 0b101) => (Instr::SRA, InstrType::R),
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
  pub fn imm(v: u32) -> u32 { v >> 20 }
  pub use crate::r::{rs1, funct3, rd};
}

pub(crate) mod s {
  pub fn imm_upper(v: u32) -> u32 { v >> 25 }
  pub use crate::r::{rs2,rs1,funct3};
  use crate::r::{rd};
  pub fn imm_lower(v: u32) -> u32 { rd(v) }
}

pub(crate) mod u {
  pub fn imm(v: u32) -> u32 { v >> 12 }
  pub use crate::r::{rd};
}

const NUM_REGS : u32 = 32;

fn run(s: String) -> Result<(), ()> {
  let mut memory = mem::create_memory(0x10000usize);
  let f = File::open(s).unwrap();
  let len = f.metadata().unwrap().len() as usize;
  assert!(len % 4 == 0);
  let mut reader = BufReader::new(f);
  let mut buffer: [u8;4] = [0,0,0,0];
  for v in 0..(len/4) {
    reader.read_exact(&mut buffer).unwrap();
    memory.write(
      v * mem::WORD_SIZE,
      u32::from_ne_bytes(buffer).to_le(),
      mem::Size::WORD)?;
  };
  unimplemented!();
}

