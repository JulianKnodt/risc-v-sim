use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::u32;
use riscv::{mem};
use riscv::program_state::{ProgramState, Status};
use riscv::sim::{normal, in_order};
use riscv::reg::Register;

#[allow(dead_code)]
enum RunType {
  Normal, Inorder, OutOfOrder,
}

fn main() {
  env::args().skip(1).for_each(|s| {
    println!("{}", s);
    run(s, RunType::Inorder).expect("Error");
  });
}

fn run(s: String, runtype: RunType) -> Result<(), ()> {
  let f = File::open(s).expect("Failed to open file");
  let len = f.metadata().expect("Could not read metadata").len() as usize;
  assert!(len % 4 == 0, "Input File is not word-aligned");
  let mut reader = BufReader::new(f);
  let mut buffer: [u8;4] = [0,0,0,0];
  let mut memory = mem::create_memory(0x10000usize); // TODO add this as an argument
  for v in 0..(len/4) {
    reader.read_exact(&mut buffer).expect("Failed to write to memory");
    memory.write(
      v * mem::WORD_SIZE,
      u32::from_ne_bytes(buffer).to_le(),
      mem::Size::WORD)?;
  };
  let ps = ProgramState::<u32> {
    regs: Register::new(32),
    mem: memory,
    status: Status::Running,
  };
  match runtype {
    RunType::Normal => normal(ps)?,
    RunType::Inorder => in_order(ps)?,
    RunType::OutOfOrder => unimplemented!(),
  };
  Ok(())
}

