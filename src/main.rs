use std::io::{BufReader, Read};
use std::u32;
use riscv::{mem};
use riscv::program_state::{ProgramState, Status};
use riscv::sim::{normal, in_order};
use riscv::reg::Register;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum RunType {
  Normal, Inorder, OutOfOrder,
}

struct Config {
  run_type: RunType,
  mem_size: usize,
}

impl Config {
  fn new() -> Config {
    Config{ run_type: RunType::Normal, mem_size: 0x10000 }
  }
}

fn main() {
  let mut config = Config::new();
  let mut files: Vec<String> = Vec::new();
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  for mut v in 0..args.len() {
    match args[v].as_str() {
      "-m" | "--mem" => {
        v += 1;
        config.mem_size = args.get(v)
          .expect("Must pass memory after --mem")
          .parse::<usize>()
          .expect("Expected Integer after --mem");
      },
      "--inorder" => config.run_type = RunType::Inorder,
      "--normal" => config.run_type = RunType::Normal,
      flag if flag.starts_with("-") => println!("Unsupported flag: {}", flag),
      f => files.push(f.to_string()),
    }
  }
  println!("{:?}", config.run_type);
  for file in files.iter() {
    println!("Running: {:?}", file);
    run(file.to_string(), &config).expect(&format!("Failed on file {:?}", file));
  };
}

fn run(s: String, c: &Config) -> Result<(), ()> {
  let f = std::fs::File::open(s).expect("Failed to open file");
  let len = f.metadata().expect("Could not read metadata").len() as usize;
  assert!(len % 4 == 0, "Input File is not word-aligned");
  let mut reader = BufReader::new(f);
  let mut buffer: [u8;4] = [0,0,0,0];
  let mut memory = mem::Memory::new(c.mem_size);
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
  match c.run_type {
    RunType::Normal => normal(ps)?,
    RunType::Inorder => in_order(ps)?,
    RunType::OutOfOrder => unimplemented!(),
  };
  Ok(())
}

