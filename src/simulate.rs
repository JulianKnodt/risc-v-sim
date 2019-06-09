use crate::mem;
use crate::reg::{Register, RegData, RegisterEntry};
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

pub(crate) const HALT: u32 = 0xfeedfeed;

#[derive(PartialEq, Debug)]
struct ProgramState<T : RegData> {
  regs: Register<T>,
  mem: mem::Memory,
  status: Status,
}


impl <T : RegData> ProgramState<T> {
  // Sign Extend
  pub fn sx(&self, reg: RegisterEntry<T>) -> T::Signed { reg.v().to_signed() }
  // Zero Extend
  pub fn zx(&self, reg: RegisterEntry<T>) -> T { reg.v() }
}

pub fn execute(m: mem::Memory) -> Result<(), ()> {
  let mut ps = ProgramState::<u32> {
    regs: Register::new(32),
    mem: m,
    status: Status::Running
  };
  while ps.status == Status::Running { ps = run_instr(ps); }
  Ok(())
}

fn run_instr<T : RegData>(mut ps: ProgramState<T>) -> ProgramState<T> {
  let raw = ps.mem.read(ps.regs.pc.as_usize(), mem::Size::WORD)
    .unwrap_or_else(|_| panic!("Failed to read instr at {:?}", ps.regs.pc));
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
      let (rs2, rs1, rd) = (rs2(raw), rs1(raw), rd(raw));
      ps.regs[rd] = RegisterEntry::Valid(match r {
        RInstr::ADD => T::from_signed(ps.sx(ps.regs[rs1]) + ps.sx(ps.regs[rs1])),
        RInstr::SUB => T::from_signed(ps.sx(ps.regs[rs1]) - ps.sx(ps.regs[rs2])),
        RInstr::SLL => ps.zx(ps.regs[rs1]) << ps.regs[rs2].v(),
        RInstr::SLT => if ps.sx(ps.regs[rs1]) < ps.sx(ps.regs[rs2]) {T::one()} else {T::zero()},
        RInstr::SLTU => if ps.zx(ps.regs[rs1]) < ps.zx(ps.regs[rs2]) {T::one()} else {T::zero()},
        RInstr::XOR => ps.zx(ps.regs[rs1]) ^ ps.zx(ps.regs[rs2]),
        RInstr::SRL => ps.zx(ps.regs[rs1]) >> ps.regs[rs2].v(),
        RInstr::SRA => T::from_signed(ps.sx(ps.regs[rs1]) >> ps.regs[rs2].v().to_signed()),
        RInstr::OR => ps.zx(ps.regs[rs1]) | ps.zx(ps.regs[rs2]),
        RInstr::AND => ps.zx(ps.regs[rs1]) & ps.zx(ps.regs[rs2]),
        RInstr::SLLI => ps.zx(ps.regs[rs1]) << T::from(rs2),
        RInstr::SRLI => ps.zx(ps.regs[rs1]) >> T::from(rs2),
        RInstr::SRAI => T::from_signed(ps.sx(ps.regs[rs1]) >> T::from(rs2).to_signed()),
      });
    },
    InstrType::I(i) => {
      use crate::instr::i::*;
      use crate::instr::IInstr;
      let (rs1, rd) = (rs1(raw), rd(raw));
      let (sx_imm, zx_imm) = (T::Signed::from(sx_imm(raw)), T::from(zx_imm(raw)));
      ps.regs[rd] = RegisterEntry::Valid(match i {
        IInstr::ADDI => T::from_signed(ps.sx(ps.regs[rs1]) + sx_imm),
        IInstr::SLTI => if ps.sx(ps.regs[rs1]) < sx_imm { T::one() } else { T::zero() },
        IInstr::SLTIU => if ps.zx(ps.regs[rs1]) < zx_imm { T::one() } else { T::zero() },
        IInstr::XORI => ps.zx(ps.regs[rs1]) ^ zx_imm,
        IInstr::ORI => ps.zx(ps.regs[rs1]) | zx_imm,
        IInstr::ANDI => ps.zx(ps.regs[rs1]) & zx_imm,
        IInstr::JALR => {
          let result = ps.regs.pc;
          ps.regs.pc = T::from_signed((ps.sx(ps.regs[rs1]) + sx_imm) & T::Signed::from(-2));
          result
        },
        IInstr::LW =>
          ps.mem.read(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm).as_usize(), mem::Size::WORD)
            .unwrap_or_else(|_| ps.regs[rd].v()),
        IInstr::LH =>
          ps.mem.read(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm).as_usize(), mem::Size::HALF)
            .unwrap_or_else(|_| ps.regs[rd].v()),
        IInstr::LB =>
          ps.mem.read(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm).as_usize(), mem::Size::BYTE)
            .unwrap_or_else(|_| ps.regs[rd].v()),
        IInstr::LHU =>
          ps.mem.read_signed::<T>(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm)
            .as_usize(), mem::Size::HALF)
            .map(|s| T::from_signed(s))
            .unwrap_or_else(|_| ps.regs[rd].v()),
        IInstr::LBU =>
          ps.mem.read_signed::<T>(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm)
          .as_usize(), mem::Size::BYTE)
            .map(|s| T::from_signed(s))
            .unwrap_or_else(|_| ps.regs[rd].v()),
        v => panic!("Unimplemented {:?}", v),
      });
    },
    InstrType::S(s) => {
      use crate::instr::s::*;
      use crate::instr::SInstr;
      let (rs1, rs2, imm) = (rs1(raw), rs2(raw), imm(raw));
      let size = match s {
        SInstr::SB => mem::Size::BYTE,
        SInstr::SH => mem::Size::HALF,
        SInstr::SW => mem::Size::WORD,
      };
      if let Err(e) = ps.mem.write((ps.regs[rs1].v() + T::from(imm)).as_usize(),
        ps.regs[rs2].v(), size) {
          println!("{:?}", e);
          ps.status = Status::Exception(Exceptions::Mem);
      };
    },
    InstrType::B(b) => {
      use crate::instr::b::*;
      use crate::instr::BInstr;
      let (rs2, rs1) = (rs2(raw), rs1(raw));
      let branch = match b {
        BInstr::BEQ => ps.regs[rs1] == ps.regs[rs2],
        BInstr::BNE => ps.regs[rs1] != ps.regs[rs2],
        BInstr::BLT => ps.sx(ps.regs[rs1]) < ps.sx(ps.regs[rs2]),
        BInstr::BGE => ps.sx(ps.regs[rs1]) >= ps.sx(ps.regs[rs2]),
        BInstr::BLTU => ps.zx(ps.regs[rs1]) < ps.zx(ps.regs[rs2]),
        BInstr::BGEU => ps.zx(ps.regs[rs1]) >= ps.zx(ps.regs[rs2]),
      };
      if branch {
        ps.regs.pc = ps.regs.pc.offset(T::Signed::from(imm(raw)))
          - T::from(mem::WORD_SIZE as u32);
      };
    },
    InstrType::U(u) => {
      use crate::instr::u::*;
      use crate::instr::UInstr;
      let (rd, imm) = (rd(raw), imm(raw));
      ps.regs[rd] = RegisterEntry::Valid(match u {
        UInstr::LUI => T::from(imm),
        UInstr::AUIPC => T::from(imm) + ps.regs.pc,
      });
    },
    InstrType::J(j) => {
      use crate::instr::j::{rd, offset};
      use crate::instr::JInstr;
      let (rd, offset) = (rd(raw), offset(raw));
      match j {
        JInstr::JAL => {
          ps.regs[rd] = RegisterEntry::Valid(ps.regs.pc);
          ps.regs.pc = ps.regs.pc.offset(T::Signed::from(offset))
            - T::from(mem::WORD_SIZE as u32);
        },
      };
    },
  };
  ps.regs[0] = RegisterEntry::Valid(T::zero());
  ps.regs.inc_pc();
  return ps;
}

