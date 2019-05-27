#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::u32;
use riscv::mem;

fn main() {
  env::args().for_each(|s| {
    run(s).expect("Error");
  });
}

fn run(s: String) -> Result<(), ()> {
  let mut memory = mem::create_memory(0x10000usize);
  let f = File::open(s).unwrap();
  let len = f.metadata().unwrap().len() as usize;
  assert!(len % 4 == 0);
  let mut reader = BufReader::new(f);
  let mut buffer: [u8;4] = [0,0,0,0];
  for v in 0..(len/4) {
    reader.read_exact(&mut buffer).unwrap();
    memory.write(
      v * mem::WORD_SIZE,
      u32::from_ne_bytes(buffer).to_le(),
      mem::Size::WORD)?;
  };
  unimplemented!();
}

