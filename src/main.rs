use std::env;
use std::fs::File;
use std::io::{Read};

fn main() {
  env::args().for_each(|s| {
    run(s).expect("Error");
  });
}

#[derive(Debug)]
struct MemOutOfBounds {}

const MEM_SIZE : usize = 10_000;
#[derive(Clone)]
struct MemBlock ([u8; MEM_SIZE]);

#[derive(Debug)]
struct RegFile {
  zero: (),
  ra: u32,
  sp: u32,
  gp: u32,
  tp: u32,
  temp1: [u32; 2],
  fp: u32,
  s1: u32,
  args: [u32; 9],
  saved: [u32; 10],
  temp2: [u32; 4],
}

#[derive(Debug)]
enum Exception {}

fn to_big_endian(u: u32) -> u32 {
  let mut out = 0u32;
  out &= (u & 0xff) << 24;
  out &= (u & 0xff00) << 8;
  out &= (u & 0xff0000) >> 8;
  out &= (u & 0xff000000) >> 24;
  out
}

const WORD_SIZE : usize = 4;
impl MemBlock {
  pub fn new() -> Self {
    MemBlock([0u8; 10_000])
  }
  pub fn write_byte(&mut self, at: usize, b: u8) -> Result<(), MemOutOfBounds> {
    if at > MEM_SIZE-1 { return Err(MemOutOfBounds{}) }
    self.0[at] = b;
    Ok(())
  }
  pub fn write_short(&mut self, at: usize, s: u16) -> Result<(), MemOutOfBounds> {
    if at > MEM_SIZE-2 { return Err(MemOutOfBounds{}) }
    self.0[at] = (s & 0xff) as u8;
    self.0[at+1] = ((s & 0xff00) >> 8) as u8;
    Ok(())
  }
  pub fn write_word(&mut self, at: usize, s: u32) -> Result<(), MemOutOfBounds> {
    if at > MEM_SIZE-WORD_SIZE { return Err(MemOutOfBounds{}) }
    self.0[at] = (s & 0xff) as u8;
    self.0[at+1] = ((s & 0xff00) >> 8) as u8;
    self.0[at+2] = ((s & 0xff0000) >> 16) as u8;
    self.0[at+3] = ((s & 0xff000000) >> 24) as u8;
    Ok(())
  }
}

fn run(s: String) -> Result<(MemBlock, RegFile), Exception> {
  let mut mem = MemBlock::new();
  let regs = [0u32; 32];
  let mut pc = 0usize;
  File::open(s).expect("Passed invalid file").bytes().fold((0, 0u32), |(counter, tmp), n|
    ((counter+1)%4, if counter == 3 {
      mem.write_word(pc, to_big_endian(tmp)).expect("Memory error");
      pc += WORD_SIZE;
      0
    } else { (tmp << 8) & (n.unwrap() as u32) }));
  unimplemented!();
}
