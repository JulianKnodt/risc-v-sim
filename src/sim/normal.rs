use crate::mem;
use crate::program_state::{ProgramState, Status, Exceptions};
use crate::reg::{RegData, RegisterEntry};
use crate::instr::{self, InstrType};

pub(crate) const HALT: u32 = 0xfeedfeed;

pub fn execute<T : RegData>(mut ps: ProgramState<T>) -> Result<ProgramState<T>, ()> {
  while ps.status == Status::Running { ps = run_instr(ps); }
  ps.regs.data.iter().enumerate().for_each(|(i,r)| {
    if r.v() != T::zero() {
      println!("[x{}: {:x}]", i, r.v())
    }
  });
  Ok(ps)
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
    instr::InstrType::R{ var: r, rs1, rs2, rd } => {
      use crate::instr::RInstr;
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
    InstrType::I{ var: i, rs1, rd, sx_imm: sx, zx_imm: zx } => {
      use crate::instr::IInstr;
      let (sx_imm, zx_imm) = (T::Signed::from(sx), T::from(zx));
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
    InstrType::S{ var: s, rs1, rs2, imm } => {
      use crate::instr::SInstr;
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
    InstrType::B{ var: b, rs1, rs2, imm } => {
      use crate::instr::BInstr;
      let branch = match b {
        BInstr::BEQ => ps.regs[rs1] == ps.regs[rs2],
        BInstr::BNE => ps.regs[rs1] != ps.regs[rs2],
        BInstr::BLT => ps.sx(ps.regs[rs1]) < ps.sx(ps.regs[rs2]),
        BInstr::BGE => ps.sx(ps.regs[rs1]) >= ps.sx(ps.regs[rs2]),
        BInstr::BLTU => ps.zx(ps.regs[rs1]) < ps.zx(ps.regs[rs2]),
        BInstr::BGEU => ps.zx(ps.regs[rs1]) >= ps.zx(ps.regs[rs2]),
      };
      if branch {
        ps.regs.pc = ps.regs.pc.offset(T::Signed::from(imm))
          - T::from(mem::WORD_SIZE as u32);
      };
    },
    InstrType::U{ var: u, rd, imm } => {
      use crate::instr::UInstr;
      ps.regs[rd] = RegisterEntry::Valid(match u {
        UInstr::LUI => T::from(imm),
        UInstr::AUIPC => T::from(imm) + ps.regs.pc,
      });
    },
    InstrType::J{ var: j, rd, offset } => {
      use crate::instr::JInstr;
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
