use std::collections::{VecDeque, BinaryHeap, HashSet, HashMap};
use crate::instr::{InstrType, decode};
use crate::program_state::{ProgramState, Status, Exceptions};
use crate::reg::{RegData};
use std::cmp::Ordering;
use crate::mem;

#[derive(Hash, PartialEq, Eq, Debug)]
enum OutputDirective<T : RegData> {
  PC(T),
  Reg(u32, T),
  Exception(Exceptions),
  MemStore(T, usize, mem::Size),
  Nop,
  Halt,
}

#[derive(Eq)]
struct OutputArtifact<T : RegData> {
  src_pc: T,
  finish: HashSet<OutputDirective<T>>
}

impl <T : RegData> Ord for OutputArtifact<T> {
  fn cmp(&self, other: &Self) -> Ordering { self.src_pc.cmp(&other.src_pc).reverse() }
}

impl <T : RegData> PartialOrd for OutputArtifact<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl <T : RegData> PartialEq for OutputArtifact<T> {
  fn eq(&self, other: &Self) -> bool { self.src_pc == other.src_pc }
}



pub fn execute<T : RegData>(mut ps: ProgramState<T>) -> Result<ProgramState<T>, ()> {
  use OutputDirective::*;
  let mut instr_queue : VecDeque<(T, InstrType, Option<T>)> = VecDeque::new();
  let mut unprocessed = BinaryHeap::new();
  while ps.status == Status::Running {
    let curr_pc = ps.regs.pc();
    (0..10)
      .map(|i| curr_pc + T::from(i * mem::WORD_SIZE as u32))
      .filter_map(|i| ps.mem.read_instr(i.as_usize()).ok().map(|raw| (raw, i)))
      .filter_map(|(raw, pc)| match decode(raw) {
        Ok(instr) => Some((instr, pc)),
        Err(_) => None,
      })
      .for_each(|(instr, pc): (InstrType, T)| {
        let dependent = instr_queue.iter().find(|(_, i, _)| instr.depends_on(i));
        match dependent {
          Some(&(max_pc,_,_)) => instr_queue.push_back((pc,instr,Some(max_pc))),
          None => instr_queue.push_back((pc, instr, None)),
        };
      });

    let (mut runnable, mut pending) = (HashMap::new(), VecDeque::new());
    instr_queue
      .into_iter()
      .for_each(|v| {
        let (pc, instr, dependency) = v;
        match dependency {
          None => { runnable.insert(pc, instr); },
          Some(dependent_pc) =>
            if runnable.contains_key(&dependent_pc) { runnable.insert(pc, instr); }
            else { pending.push_back(v); },
        };
      });
    instr_queue = pending;

    runnable
      .into_iter()
      .map(|(pc, instr)| OutputArtifact{
        src_pc: pc,
        finish: OutputDirective::from(pc, instr, &ps),
      })
      .for_each(|v| unprocessed.push(v));

    while let Some(artifact) = unprocessed.pop() {
      if artifact.src_pc == ps.regs.pc() {
        artifact.finish
          .iter()
          .for_each(|directive| {
            println!("{:?}", directive);
            match directive {
              PC(new_pc) => ps.regs.assign_pc(*new_pc),
              Exception(e) => ps.status = Status::Exception(*e),
              Reg(rd, val) => ps.regs.force_assign(*rd, *val),
              MemStore(val, loc, sz) => if let Err(_) = ps.mem.write(*loc, *val, *sz) {
                ps.status = Status::Exception(Exceptions::Mem)
              },
              Halt => ps.status = Status::Done,
              Nop => (),
            };
          });
        if ps.status != Status::Running { break }
        ps.regs.inc_pc();
      } else {
        unprocessed.push(artifact);
        break
      }
    }
  };
  Ok(ps)
}

impl <T : RegData> OutputDirective<T> {
  // takes an instr and pc and returns a set of commands to run in random order
  fn from(pc: T, instr: InstrType, ps: &ProgramState<T>) -> HashSet<Self> {
    use crate::instr::InstrType::*;
    use crate::instr::{RInstr, IInstr, BInstr, JInstr, SInstr, UInstr};
    use OutputDirective::*;
    let mut out = HashSet::new();
    let action = match instr {
      R{ var, rs1, rs2, rd } => Reg(rd, match var {
        RInstr::ADD => T::from_signed(ps.sx(ps.regs[rs1]) + ps.sx(ps.regs[rs2])),
        RInstr::SUB => T::from_signed(ps.sx(ps.regs[rs1]) - ps.sx(ps.regs[rs2])),
        RInstr::SLL => ps.zx(ps.regs[rs1]) << ps.regs[rs2],
        RInstr::SLT => if ps.sx(ps.regs[rs1]) < ps.sx(ps.regs[rs2]) {T::one()} else {T::zero()},
        RInstr::SLTU => if ps.zx(ps.regs[rs1]) < ps.zx(ps.regs[rs2]) {T::one()} else {T::zero()},
        RInstr::XOR => ps.zx(ps.regs[rs1]) ^ ps.zx(ps.regs[rs2]),
        RInstr::SRL => ps.zx(ps.regs[rs1]) >> ps.regs[rs2],
        RInstr::SRA => T::from_signed(ps.sx(ps.regs[rs1]) >> ps.regs[rs2].to_signed()),
        RInstr::OR => ps.zx(ps.regs[rs1]) | ps.zx(ps.regs[rs2]),
        RInstr::AND => ps.zx(ps.regs[rs1]) & ps.zx(ps.regs[rs2]),
        RInstr::SLLI => ps.zx(ps.regs[rs1]) << T::from(rs2),
        RInstr::SRLI => ps.zx(ps.regs[rs1]) >> T::from(rs2),
        RInstr::SRAI => T::from_signed(ps.sx(ps.regs[rs1]) >> T::from(rs2).to_signed()),
      }),
      I{ var, rs1, rd, sx_imm, zx_imm } => {
        let (sx_imm, zx_imm) = (T::Signed::from(sx_imm), T::from(zx_imm));
        match var {
          IInstr::ADDI => Reg(rd, T::from_signed(ps.sx(ps.regs[rs1]) + sx_imm)),
          IInstr::SLTI =>
            Reg(rd, if ps.sx(ps.regs[rs1]) < sx_imm { T::one() } else { T::zero() }),
          IInstr::SLTIU =>
            Reg(rd, if ps.zx(ps.regs[rs1]) < zx_imm { T::one() } else { T::zero() }),
          IInstr::XORI => Reg(rd, ps.zx(ps.regs[rs1]) ^ zx_imm),
          IInstr::ORI => Reg(rd, ps.zx(ps.regs[rs1]) | zx_imm),
          IInstr::ANDI => Reg(rd, ps.zx(ps.regs[rs1]) & zx_imm),
          IInstr::JALR => {
            out.insert(PC(T::from_signed((ps.sx(ps.regs[rs1]) + sx_imm) & T::Signed::from(-2))));
            Reg(rd, pc)
          },
          IInstr::LW =>
            ps.mem.read(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm).as_usize(), mem::Size::WORD)
              .map(|t| Reg(rd, t))
              .unwrap_or(Exception(Exceptions::Mem)),
          IInstr::LH =>
            ps.mem.read(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm).as_usize(), mem::Size::HALF)
              .map(|t| Reg(rd, t))
              .unwrap_or(Exception(Exceptions::Mem)),
          IInstr::LB =>
            ps.mem.read(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm).as_usize(), mem::Size::BYTE)
              .map(|t| Reg(rd, t))
              .unwrap_or(Exception(Exceptions::Mem)),
          IInstr::LHU =>
            ps.mem.read_signed(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm)
              .as_usize(), mem::Size::HALF)
              .map(|s| T::from_signed(s))
              .map(|t| Reg(rd, t))
              .unwrap_or(Exception(Exceptions::Mem)),
          IInstr::LBU =>
            ps.mem.read_signed(T::from_signed(ps.sx(ps.regs[rs1])+sx_imm)
            .as_usize(), mem::Size::BYTE)
              .map(|s| T::from_signed(s))
              .map(|t| Reg(rd, t))
              .unwrap_or(Exception(Exceptions::Mem)),
          v => panic!("Unimplemented {:?}", v),
        }
      },
      S{ var, rs1, rs2, imm } => {
        let size = match var {
          SInstr::SB => mem::Size::BYTE,
          SInstr::SH => mem::Size::HALF,
          SInstr::SW => mem::Size::WORD,
        };
        MemStore(ps.regs[rs2], (ps.regs[rs1] + T::from(imm)).as_usize(), size)
      },
      B{ var, rs1, rs2, imm } => {
        let branch = match var {
          BInstr::BEQ => ps.regs[rs1] == ps.regs[rs2],
          BInstr::BNE => ps.regs[rs1] != ps.regs[rs2],
          BInstr::BLT => ps.sx(ps.regs[rs1]) < ps.sx(ps.regs[rs2]),
          BInstr::BGE => ps.sx(ps.regs[rs1]) >= ps.sx(ps.regs[rs2]),
          BInstr::BLTU => ps.zx(ps.regs[rs1]) < ps.zx(ps.regs[rs2]),
          BInstr::BGEU => ps.zx(ps.regs[rs1]) >= ps.zx(ps.regs[rs2]),
        };
        if branch { PC(pc.offset(T::Signed::from(imm)) - T::from(mem::WORD_SIZE as u32)) }
        else { Nop }
      },
      U{ var, rd, imm } => Reg(rd, match var {
        UInstr::LUI => T::from(imm),
        UInstr::AUIPC => T::from(imm) + pc,
      }),
      J{ var, rd, offset } => match var {
        JInstr::JAL => {
          out.insert(Reg(rd, pc));
          PC(pc.offset(T::Signed::from(offset)) - T::from(mem::WORD_SIZE as u32))
        },
      },
      InstrType::Halt => OutputDirective::Halt,
    };
    out.insert(action);
    out
  }
}















