#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::u32;
use riscv::{mem, simulate};

fn main() {
  env::args().skip(1).for_each(|s| {
    println!("{}", s);
    run(s).expect("Error");
  });
}

fn run(s: String) -> Result<(), ()> {
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
  simulate::execute(memory)
}

