use crate::mem;
use crate::reg::{RegData};
use crate::program_state::{ProgramState, Status, Exceptions, HALT};
use crate::instr::{self, InstrType, RInstr, IInstr, BInstr, JInstr, SInstr, UInstr};

// Pipeline elements can either be exceptions or instructions
type PipelineEntry = Result<u32, Exceptions>;

#[derive(Clone, Copy, Debug)]
struct Pipeline([PipelineEntry;5]);
impl std::ops::Index<Phases> for Pipeline {
  type Output = PipelineEntry;
  fn index(&self, p: Phases) -> &PipelineEntry { &self.0[p as usize] }
}

impl std::ops::IndexMut<Phases> for Pipeline {
  fn index_mut(&mut self, p: Phases) -> &mut PipelineEntry { &mut self.0[p as usize] }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phases {
  IF=0, ID=1, EX=2, MEM=3, WB=4,
}

pub fn in_order<T : RegData>(mut ps: ProgramState<T>) -> Result<ProgramState<T>, ()> {
  let mut p: Pipeline = Pipeline([Ok(0); 5]);
  while ps.status == Status::Running {
    ps = ps.run_phase(&p, Phases::WB)
      .run_phase(&p, Phases::MEM)
      .run_phase(&p, Phases::EX)
      .run_phase(&p, Phases::ID);
    ps.run_if_phase(&mut p);
    ps.regs.inc_pc();
  };
  Ok(ps)
}

impl <T: RegData>ProgramState<T> {
  fn run_phase(mut self, p: &Pipeline, phase: Phases) -> ProgramState<T> {
    let instr = match p[phase] {
      Ok(HALT) => if phase == Phases::WB {
          self.status = Status::Done;
          return self;
        } else { return self },
      Ok(raw) => {
        instr::decode(raw)
      },
      Err(_) if phase != Phases::WB => return self,
      Err(e) => {
        self.status = Status::Exception(e);
        return self;
      },
    };
    match phase {
      Phases::IF => panic!("Unexpected run_phase() with Phases::IF, use run_if_phase instead"),
      Phases::ID => {
        match instr {
          InstrType::I{ var: IInstr::JALR, rs1, rd, sx_imm, .. } => {
            let curr_pc = self.regs.pc.v();
            let sx = T::Signed::from(sx_imm);
            self.regs.pc.write(
              T::from_signed(self.sx(self.regs[rs1])+sx & T::Signed::from(-2))
            );
            self.regs[rd].write(curr_pc);
          },
          InstrType::B{ var, rs1, rs2, imm } => {
            let branch = match var {
              BInstr::BEQ => self.regs[rs1].v() == self.regs[rs2].v(),
              BInstr::BNE => self.regs[rs1].v() != self.regs[rs2].v(),
              BInstr::BLT => self.sx(self.regs[rs1]) < self.sx(self.regs[rs2]),
              BInstr::BGE => self.sx(self.regs[rs1]) >= self.sx(self.regs[rs2]),
              BInstr::BLTU => self.zx(self.regs[rs1]) < self.zx(self.regs[rs2]),
              BInstr::BGEU => self.zx(self.regs[rs1]) >= self.zx(self.regs[rs2]),
            };
            if branch {
              self.regs.pc.write(self.regs.pc.v().offset(T::Signed::from(imm))
                - T::from(mem::WORD_SIZE as u32));
            };
          },
          InstrType::J{ var, rd, offset } => match var {
            JInstr::JAL => {
              let curr_pc = self.regs.pc.v();
              self.regs[rd].write(curr_pc);
              self.regs.pc.write(self.regs.pc.v().offset(T::Signed::from(offset))
                - T::from(mem::WORD_SIZE as u32));
            },
          },
          _ => (),
        };
      },
      Phases::EX => {
        unimplemented!();
      },
      Phases::MEM => {
        unimplemented!();
      },
      Phases::WB => match instr {
        InstrType::R{ rd, .. } => self.regs[rd].writeback(),
        InstrType::I{ var, rd, .. } => {
          self.regs[rd].writeback();
          if let IInstr::JALR = var { self.regs.pc.writeback() }
        },
        InstrType::S{ .. } => unimplemented!(), // TODO actually write to memory here
        InstrType::U{ rd, .. } => self.regs[rd].writeback(),
        InstrType::J{ rd, .. } => {
          self.regs[rd].writeback();
          self.regs.pc.writeback()
        },
        InstrType::B{ .. } => self.regs.pc.writeback(),
      },
    };
    self
  }
  fn run_if_phase(&mut self, p: &mut Pipeline) {
    p[Phases::IF] = Ok(self.mem.read(self.regs.pc.as_usize(), mem::Size::WORD)
      .expect("Failed to read instruction"));
  }
}



