use crate::mem;
use crate::reg::{RegData};
use crate::program_state::{ProgramState, Status, Exceptions};
use crate::instr::{self, InstrType, RInstr, IInstr, BInstr, JInstr, SInstr, UInstr};

// Pipeline elements can either be exceptions or instructions
#[derive(Clone, Copy, Debug, PartialEq)]
enum PipelineEntry { Empty, Exc(Exceptions), Instr(u32), }

const PIPE_SIZE: usize = 5;
#[derive(Clone, Copy, Debug)]
struct Pipeline([PipelineEntry;PIPE_SIZE]);
impl std::ops::Index<Phases> for Pipeline {
  type Output = PipelineEntry;
  fn index(&self, p: Phases) -> &PipelineEntry { &self.0[p as usize] }
}

impl Pipeline {
  fn shift(&mut self) {
    (0..PIPE_SIZE-1).rev().for_each(|v| self.0[v+1] = self.0[v]);
    self.0[0] = PipelineEntry::Empty;
  }
  // TODO only if there are no jumps ahead?
  fn done(&self) -> bool {
    self.0.iter().any(|&v| v == PipelineEntry::Instr(InstrType::halt_val()))
  }
}

impl std::ops::IndexMut<Phases> for Pipeline {
  fn index_mut(&mut self, p: Phases) -> &mut PipelineEntry { &mut self.0[p as usize] }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phases { IF=0, ID=1, EX=2, MEM=3, WB=4, }

pub fn in_order<T : RegData>(mut ps: ProgramState<T>) -> Result<ProgramState<T>, ()> {
  let mut p: Pipeline = Pipeline([PipelineEntry::Empty; PIPE_SIZE]);
  while ps.status == Status::Running {
    ps = ps.run_phase(&mut p, Phases::WB)
      .run_phase(&mut p, Phases::MEM)
      .run_phase(&mut p, Phases::EX)
      .run_phase(&mut p, Phases::ID);
    ps.run_if_phase(&mut p);
    if !p.done() { ps.regs.inc_pc(); }
    p.shift();
  };
  println!("{}", ps.regs);
  Ok(ps)
}

impl <T: RegData>ProgramState<T> {
  fn run_phase(mut self, p: &mut Pipeline, phase: Phases) -> ProgramState<T> {
    use PipelineEntry::*;
    let instr = match p[phase] {
      Empty => return self,
      Instr(raw) => instr::decode(raw).unwrap(),
      Exc(_) if phase != Phases::WB => return self,
      Exc(e) => {
        self.status = Status::Exception(e);
        return self;
      },
    };
    match phase {
      Phases::IF => panic!("Unexpected run_phase() with Phases::IF, use run_if_phase instead"),
      Phases::ID => match instr {
        InstrType::I{ var: IInstr::JALR, rs1, rd, sx_imm, .. } => {
          let curr_pc = self.regs.pc();
          let sx = T::Signed::from(sx_imm);
          self.regs.assign_pc(
            T::from_signed(self.sx(self.regs[rs1])+sx & T::Signed::from(-2))
          );
          self.regs.assign(rd, curr_pc);
        },
        InstrType::B{ var, rs1, rs2, imm } => {
          let branch = match var {
            BInstr::BEQ => self.regs[rs1] == self.regs[rs2],
            BInstr::BNE => self.regs[rs1] != self.regs[rs2],
            BInstr::BLT => self.sx(self.regs[rs1]) < self.sx(self.regs[rs2]),
            BInstr::BGE => self.sx(self.regs[rs1]) >= self.sx(self.regs[rs2]),
            BInstr::BLTU => self.zx(self.regs[rs1]) < self.zx(self.regs[rs2]),
            BInstr::BGEU => self.zx(self.regs[rs1]) >= self.zx(self.regs[rs2]),
          };
          if branch {
            self.regs.assign_pc(self.regs.pc().offset(T::Signed::from(imm))
              - T::from(mem::WORD_SIZE as u32));
          };
        },
        InstrType::J{ var, rd, offset } => match var {
          JInstr::JAL => {
            let curr_pc = self.regs.pc();
            self.regs.assign(rd, curr_pc);
            self.regs.assign_pc(self.regs.pc().offset(T::Signed::from(offset))
              - T::from(mem::WORD_SIZE as u32));
          },
        },
        _ => (),
      },
      Phases::EX => match instr {
        InstrType::R{ var, rs1, rs2, rd } => {
          let result = match var {
            RInstr::ADD => T::from_signed(self.sx(self.regs[rs1]) + self.sx(self.regs[rs2])),
            RInstr::SUB => T::from_signed(self.sx(self.regs[rs1]) - self.sx(self.regs[rs2])),
            RInstr::SLL => self.zx(self.regs[rs1]) << self.regs[rs2],
            RInstr::SLT =>
              if self.sx(self.regs[rs1]) < self.sx(self.regs[rs2]) {T::one()} else {T::zero()},
            RInstr::SLTU =>
              if self.zx(self.regs[rs1]) < self.zx(self.regs[rs2]) {T::one()} else {T::zero()},
            RInstr::XOR => self.zx(self.regs[rs1]) ^ self.zx(self.regs[rs2]),
            RInstr::SRL => self.zx(self.regs[rs1]) >> self.regs[rs2],
            RInstr::SRA => T::from_signed(self.sx(self.regs[rs1]) >> self.regs[rs2].to_signed()),
            RInstr::OR => self.zx(self.regs[rs1]) | self.zx(self.regs[rs2]),
            RInstr::AND => self.zx(self.regs[rs1]) & self.zx(self.regs[rs2]),
            RInstr::SLLI => self.zx(self.regs[rs1]) << T::from(rs2),
            RInstr::SRLI => self.zx(self.regs[rs1]) >> T::from(rs2),
            RInstr::SRAI => T::from_signed(self.sx(self.regs[rs1]) >> T::from(rs2).to_signed()),
          };
          self.regs.assign(rd, result);
        },
        InstrType::I{ var, rs1, rd, sx_imm, zx_imm } => {
          let (sx_imm, zx_imm) = (T::Signed::from(sx_imm), T::from(zx_imm));
          let result = match var {
            IInstr::ADDI => Some(T::from_signed(self.sx(self.regs[rs1]) + sx_imm)),
            IInstr::SLTI =>
              Some(if self.sx(self.regs[rs1]) < sx_imm {T::one()} else {T::zero()}),
            IInstr::SLTIU =>
              Some(if self.zx(self.regs[rs1]) < zx_imm {T::one()} else {T::zero()}),
            IInstr::XORI => Some(self.zx(self.regs[rs1]) ^ zx_imm),
            IInstr::ORI => Some(self.zx(self.regs[rs1]) | zx_imm),
            IInstr::ANDI => Some(self.zx(self.regs[rs1]) & zx_imm),
            _ => None,
          };
          if let Some(result) = result { self.regs.assign(rd, result); };
        },
        InstrType::U{ var, rd, imm } => {
          let result = match var {
            UInstr::LUI => T::from(imm),
            UInstr::AUIPC => T::from(imm) + self.regs.pc(),
          };
          self.regs.assign(rd, result);
        },
        _ => (),
      },
      Phases::MEM => match instr {
        InstrType::I { var, rd, rs1, sx_imm, .. } => {
          let sx= T::Signed::from(sx_imm);
          let result = match var {
            IInstr::LW =>
              self.mem.read(
                T::from_signed(self.sx(self.regs[rs1])+sx).as_usize(), mem::Size::WORD),
            IInstr::LH =>
              self.mem.read(
                T::from_signed(self.sx(self.regs[rs1])+sx).as_usize(), mem::Size::HALF),
            IInstr::LB =>
              self.mem.read(
                T::from_signed(self.sx(self.regs[rs1])+sx).as_usize(), mem::Size::BYTE),

            IInstr::LHU =>
              self.mem.read_signed(
                T::from_signed(self.sx(self.regs[rs1])+sx).as_usize(), mem::Size::HALF)
              .map(|s| T::from_signed(s)),
            IInstr::LBU =>
              self.mem.read_signed(
                T::from_signed(self.sx(self.regs[rs1])+sx).as_usize(), mem::Size::BYTE)
              .map(|s| T::from_signed(s)),
            _ => return self,
          };
          match result {
            Ok(v) => self.regs.assign(rd, v),
            Err(_) => p[phase] = PipelineEntry::Exc(Exceptions::Mem),
          };
        },
        InstrType::S { var, rs1, rs2, imm } => {
          let sz = match var {
            SInstr::SW => mem::Size::WORD,
            SInstr::SH => mem::Size::HALF,
            SInstr::SB => mem::Size::BYTE,
          };
          let result = self.mem
            .queue_write((self.regs[rs1] + T::from(imm)).as_usize(), self.regs[rs2], sz);
          if let Err(()) = result {
            p[phase] = PipelineEntry::Exc(Exceptions::Mem);
          };
        },
        _ => (),
      },
      Phases::WB => match instr {
        InstrType::Halt => self.status = Status::Done,
        InstrType::S{ .. } => assert!(self.mem.complete_write().is_ok()),
        InstrType::B{ .. } => (),
        InstrType::J{ rd, .. } | InstrType::I{ rd, .. }
          | InstrType::R{ rd, .. } | InstrType::U{ rd, .. } =>
          assert!(self.regs.writeback(rd)),
      },
    };
    self
  }
  fn run_if_phase(&mut self, p: &mut Pipeline) {
    p[Phases::IF] = if p.done() { PipelineEntry::Empty }
                    else {
                      PipelineEntry::Instr(self.mem.read_instr(self.regs.pc().as_usize())
                        .expect("Failed to read instruction"))
                    };
  }
}



