#![allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum RInstr {
  SLLI, SRLI, SRAI, ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND,
  // M-extension
  // MUL, MULH, MULHSU, MULHU, DIV, DIVU, REM, REMU,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IInstr {
  JALR, LB, LH, LW, LBU, LHU, ADDI, SLTI, SLTIU, XORI, ORI, ANDI,

  // unimplemented
  ECALL, EBREAK, CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BInstr {
  BEQ, BNE, BLT, BGE, BLTU, BGEU,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum JInstr {
  JAL,
}

// not sure what to do with these
#[derive(Clone, Copy, Debug)]
pub(crate) enum Instr {
  FENCE, FENCEI,
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
  R{ var: RInstr, rs1: u32, rs2: u32, rd: u32 },
  I{ var: IInstr, rs1: u32, rd: u32, sx_imm: i32, zx_imm: u32 },
  S{ var: SInstr, rs1: u32, rs2: u32, imm: u32 },
  B{ var: BInstr, rs1: u32, rs2: u32, imm: i32 },
  U{ var: UInstr, rd: u32, imm: u32 },
  J{ var: JInstr, rd: u32, offset: i32 } ,

  Halt,
}

impl InstrType {
  pub fn r(r: RInstr, v: u32) -> InstrType {
    use self::r::*;
    InstrType::R{ var: r, rs1: rs1(v), rs2: rs2(v), rd: rd(v) }
  }
  pub fn i(i: IInstr, v: u32) -> InstrType {
    use self::i::*;
    InstrType::I{ var: i, rs1: rs1(v), rd: rd(v), sx_imm: sx_imm(v), zx_imm: zx_imm(v) }
  }
  pub fn s(s: SInstr, v: u32) -> InstrType {
    use self::s::*;
    InstrType::S{ var: s, rs1: rs1(v), rs2: rs2(v), imm: imm(v) }
  }
  pub fn b(b: BInstr, v: u32) -> InstrType {
    use self::b::*;
    InstrType::B{ var: b, rs1: rs1(v), rs2: rs2(v), imm: imm(v) }
  }
  pub fn u(u: UInstr, v: u32) -> InstrType {
    use self::u::*;
    InstrType::U{ var: u, rd: rd(v), imm: imm(v) }
  }
  pub fn j(j: JInstr, v: u32) -> InstrType {
    use self::j::*;
    InstrType::J{ var: j, rd: rd(v), offset: offset(v) }
  }
  pub const fn halt_val() -> u32 { 0xfeedfeedu32 }
  pub fn depends_on(&self, on: &InstrType) -> bool {
    use InstrType::*;
    match self {
      Halt => false,
      U{ var: UInstr::AUIPC, .. } => match on {
        J{..} | B{ .. } | I{ var: IInstr::JALR, .. } | Halt => true,
        _ => false,
      },
      J{ .. } | U{ .. } => false,
      R{ rs1, rs2, ..} | S{rs1,rs2, ..} | B{ rs1, rs2, ..} => match on {
        S{..} | B{..} => false,
        R{rd, ..} | I{rd, ..} | U{rd, ..} | J{rd, ..} => rd == rs1 || rd == rs2,
        Halt => true,
      }
      I{ var: IInstr::JALR, rs1, .. } => match on {
        J{..} | B{ .. } | I{ var: IInstr::JALR, .. } => true,
        S{..} => false,
        R{rd, ..} | I{rd, ..} | U{rd, ..} => rd == rs1,
        Halt => true,
      },
      I{ rs1, .. } => match on {
        // TODO loads can be dependent depending on where the stores are
        S{..} | B{..} => false,
        R{rd, ..} | I{rd, ..} | U{rd, ..} | J{rd, ..} => rd == rs1,
        Halt => true,
      }
    }
  }
}

pub(crate) fn decode(instr: u32) -> Result<InstrType, String> {
  let v = instr; // just for aliasing
  if v == InstrType::halt_val() { return Ok(InstrType::Halt) }
  let instr = match opcode(v) {
    0b1101111 => InstrType::j(JInstr::JAL, v),
    0b0110111 => InstrType::u(UInstr::LUI, v),
    0b0010111 => InstrType::u(UInstr::AUIPC, v),
    0b1100111 if i::funct3(instr) == 0 => InstrType::i(IInstr::JALR, v),
    0b1100011 => match s::funct3(instr) {
      0b000 => InstrType::b(BInstr::BEQ, v),
      0b001 => InstrType::b(BInstr::BNE, v),
      0b100 => InstrType::b(BInstr::BLT, v),
      0b101 => InstrType::b(BInstr::BGE, v),
      0b110 => InstrType::b(BInstr::BLTU, v),
      0b111 => InstrType::b(BInstr::BGEU, v),
      funct3 => return Err(format!("Unexpected funct3 for opcode 0b1100011 : {}", funct3)),
    },
    0b0000011 => match i::funct3(instr) {
      0b000 => InstrType::i(IInstr::LB, v),
      0b001 => InstrType::i(IInstr::LH, v),
      0b010 => InstrType::i(IInstr::LW, v),
      0b100 => InstrType::i(IInstr::LBU, v),
      0b101 => InstrType::i(IInstr::LHU, v),
      funct3 => return Err(format!("Unexpected funct3 for opcode 0b0000011: {}", funct3)),
    },
    0b0100011 => match s::funct3(instr) {
      0b000 => InstrType::s(SInstr::SB, v),
      0b001 => InstrType::s(SInstr::SH, v),
      0b010 => InstrType::s(SInstr::SW, v),
      funct3 => return Err(format!("Unexpected funct3 for opcode 0b0100011: {}", funct3)),
    },
    0b0010011 => match i::funct3(instr) {
      0b000 => InstrType::i(IInstr::ADDI, v),
      0b010 => InstrType::i(IInstr::SLTI, v),
      0b011 => InstrType::i(IInstr::SLTIU, v),
      0b100 => InstrType::i(IInstr::XORI, v),
      0b110 => InstrType::i(IInstr::ORI, v),
      0b111 => InstrType::i(IInstr::ANDI, v),
      0b001 => InstrType::r(RInstr::SLLI, v),
      0b101 if r::funct7(instr) == 0 => InstrType::r(RInstr::SRLI, v),
      0b101 => InstrType::r(RInstr::SRAI, v),
      funct3 => return Err(format!("Unexpected funct3 for opcode 0b0010111: {}", funct3)),
    },
    0b0110011 => match (r::funct7(instr), r::funct3(instr)) {
      (0, 0b000) => InstrType::r(RInstr::ADD, v),
      (32, 0b000)=> InstrType::r(RInstr::SUB, v),
      (0, 0b001) => InstrType::r(RInstr::SLL, v),
      (0, 0b010) => InstrType::r(RInstr::SLT, v),
      (0, 0b011) => InstrType::r(RInstr::SLTU, v),
      (0, 0b100) => InstrType::r(RInstr::XOR, v),
      (0, 0b101) => InstrType::r(RInstr::SRL, v),
      (32, 0b101) => InstrType::r(RInstr::SRA, v),
      (0, 0b110) => InstrType::r(RInstr::OR, v),
      (0, 0b111) => InstrType::r(RInstr::AND, v),
      // Multiplication extension
      //(1, 0b000) => InstrType::R(RInstr::MUL),
      //(1, 0b001) => InstrType::R(RInstr::MULH),
      //(1, 0b010) => InstrType::R(RInstr::MULHSU),
      //(1, 0b011) => InstrType::R(RInstr::MULHU),
      //(1, 0b100) => InstrType::R(RInstr::DIV),
      //(1, 0b101) => InstrType::R(RInstr::DIVU),
      //(1, 0b110) => InstrType::R(RInstr::REM),
      //(1, 0b111) => InstrType::R(RInstr::REMU),
      (f7, f3) =>
        return Err(format!("Unexpected funct7 & funct3 for opcode 0b0110011: {}, {}", f7, f3)),
    },
    0b1110011 => match i::funct3(instr) {
      0b000 => match i::zx_imm(instr) {
        0 => InstrType::i(IInstr::ECALL, v),
        1 => InstrType::i(IInstr::EBREAK, v),
        v =>
          return Err(format!("Unexpected immediate for opcode: 0b1110011, funct3: 0b000, {}",v)),
      },
      0b001 => InstrType::i(IInstr::CSRRW, v),
      0b010 => InstrType::i(IInstr::CSRRS, v),
      0b011 => InstrType::i(IInstr::CSRRC, v),
      0b101 => InstrType::i(IInstr::CSRRW, v),
      0b110 => InstrType::i(IInstr::CSRRS, v),
      0b111 => InstrType::i(IInstr::CSRRC, v),
      v => return Err(format!("Unexpected funct3 for opcode: 0b1110011, funct3: {}", v)),
    },
    v => return Err(format!("Unexpected Opcode {:b} for instr {:b}", v, instr)),
  };
  Ok(instr)
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
  use crate::instr::r::rd;
  pub fn imm(v: u32) -> u32 {
    ((v >> 25) << 5) | rd(v)
  }
}

pub(crate) mod b {
  pub use crate::instr::s::{rs2, rs1, funct3};
  use crate::instr::s::imm as s_imm;
  const SIGN_MASK: u32 = 0xFFFFF000;
  pub fn imm(v: u32) -> i32 {
    let s = s_imm(v);
    let sign_bit = s >> 11;
    let out = ((s & 0b1) << 11) |
      (((v >> 25) & 0b111111) << 5) |
      (((v >> 8) & 0b11111) << 1);
    unsafe {
      std::mem::transmute::<u32, i32>(
        if sign_bit == 1 { SIGN_MASK | out } else { out }
      )
    }
  }

  #[test]
  fn b_imm_test() {
    let v: u32 = 0b1111111_00000_00000_000_11111_0000000;
    assert_eq!(imm(v), -2);

    let v: u32 = 0b0111111_00000_00000_000_11111_0000000;
    assert_eq!(imm(v), 0b0111111111110);
    assert_eq!(imm(v) >> 12, 0);

    let v: u32 = 0b1111111_00000_00000_000_11110_0000000;
    assert_eq!(imm(v) & 0x1FFF, 0b1011111111110);
    assert_eq!(imm(v) >> 12, -1);

    let v: u32 = 0b1000000_00000_00000_000_11111_0000000;
    assert_eq!(imm(v) & 0xFFF, 0b100000011110, "0b{:b}", imm(v));
    assert_eq!(imm(v) >> 12, -1);

    let v: u32 = 0b1111111_00000_00000_000_00001_0000000;
    assert_eq!(imm(v) & 0xFFF, 0b111111100000);
    assert_eq!(imm(v) >> 12, -1)
  }
}

pub(crate) mod u {
  pub fn imm(v: u32) -> u32 { v & 0xfffff000 }
  pub use crate::instr::r::{rd};
}

pub(crate) mod j {
  pub use crate::instr::r::rd;
  const SIGN_MASK: u32 = 0xFFF00000;
  pub fn offset(v: u32) -> i32 {
    let sign_bit = v >> 31;
    let  v = (((v >> 20) & 0b1) << 11) | ((v >> 20) & 0x7fe) | (v & 0xFF000);
    unsafe {
      std::mem::transmute::<u32, i32>(
        if sign_bit == 1 { SIGN_MASK | v } else { v }
      )
    }
  }

  #[test]
  fn test_imm() {
    let v = 0b1_1111111111_1_11111111_000000000000;
    assert_eq!(offset(v), -2, "0b{:b}",  offset(v));

    let v = 0b0_1111111111_1_11111111_000000000000;
    assert_eq!(offset(v), 0x7FFFF << 1, "0b{:b}",  offset(v));
    assert_eq!(offset(v) >> 20, 0);

    let v = 0b1_0000000000_1_11111111_000000000000;
    assert_eq!(offset(v) & 0xFFFFF, 0b11111111_1_00000000000);
    assert_eq!(offset(v) >> 20, -1, "{:b}", offset(v) >> 20);

    let v = 0b1_1111111111_0_11111111_000000000000;
    assert_eq!(offset(v) & 0xFFFFF, 0b11111111_0_1111111111_0, "0b{:b}",  offset(v) & 0xFFFFF);
    assert_eq!(offset(v) >> 20, -1, "{:b}", offset(v) >> 20);

    let v = 0b1_1111111111_1_00000000_000000000000;
    assert_eq!(offset(v) & 0xFFFFF, 0b00000000_1_1111111111_0, "0b{:b}",  offset(v) & 0xFFFFF);
    assert_eq!(offset(v) >> 20, -1, "{:b}", offset(v) >> 20);
  }
}







