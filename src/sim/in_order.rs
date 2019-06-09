use crate::mem;
use crate::reg::{RegData};
use crate::program_state::{ProgramState, Status, Exceptions};
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

#[derive(Clone, Copy, Debug)]
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
  };
  Ok(ps)
}

impl <T: RegData>ProgramState<T> {
  fn run_phase(mut self, p: &Pipeline, phase: Phases) -> ProgramState<T> {
    let instr = match p[phase] {
      Ok(raw) => instr::decode(raw),
      Err(_) => return self,
    };
    match phase {
      Phases::IF => panic!("Unexpected run_phase() with Phases::IF, use run_if_phase instead"),
      Phases::ID => {
        match instr {
          InstrType::I{ var: IInstr::JALR, rs1: rs1, rd: rd, sx_imm: sx, zx_imm: zx } => {

          },
          InstrType::B{ .. } => {},
          InstrType::J{ .. } => {},
          v => (),
        };
      },
      Phases::EX => {

      },
      Phases::MEM => {

      },
      Phases::WB => {

      },
    };
    self
  }
  fn run_if_phase(&mut self, p: &mut Pipeline) {
    let instr = self.mem.read(self.regs.pc.as_usize(), mem::Size::WORD)
      .expect("Failed to read instruction");
    p[Phases::IF] = Ok(instr);
  }
}



