use crate::reg::RegData;
use std::collections::VecDeque;
use std::ops::Range;
use std::default::Default;

pub const WORD_SIZE: usize = 4;
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Size {
  DOUBLE, // 8 bytes
  WORD, // 4 bytes
  HALF, // 2 bytes
  BYTE, // 1 byte
}

#[derive(PartialEq, Clone, Debug)]
pub struct Memory <T : RegData> {
  pub data: Vec<u8>,
  size: usize,
  write_queue: VecDeque<(usize, T, Size)>,
}

pub struct MemView <'a, T : RegData> {
  range: Range<usize>,
  m: &'a Memory<T>,
}

impl <T : RegData> Memory<T> {
  pub fn new(size: usize) -> Memory<T> {
    Memory { data: vec![0; size], size: size, write_queue: VecDeque::new(), }
  }
  pub fn write(&mut self, loc: usize, data: T, s: Size) -> Result<(), &str> {
    if loc > self.size { return Err("Mem write out of bounds") };
    let bytes = data.to_le_bytes();
    match s {
      Size::BYTE => (0..1),
      Size::HALF => (0..2),
      Size::WORD => (0..4),
      Size::DOUBLE if T::BYTE_SIZE < 8 => return Err("Not sufficient size to write word"),
      Size::DOUBLE => (0..8)
    }.for_each(|i| self.data[loc + i] = bytes[i]);
    Ok(())
  }
  pub fn read(&self, loc: usize, s: Size) -> Result<T, &str> {
    if loc > self.size { return Err("mem.read() out of bounds") };
    let mut bytes : [u8; 4] = Default::default();
    match s {
      Size::BYTE => (0..1),
      Size::HALF => (0..2),
      Size::WORD => (0..4),
      Size::DOUBLE if T::BYTE_SIZE < 8 => return Err("Not sufficient size to read double"),
      Size::DOUBLE => {
        let mut bytes: [u8; 8] = Default::default();
        (0..8).for_each(|i| bytes[i] = self.data[loc+1]);
        return Ok(T::from_le_bytes(Box::new(bytes)));
      },
    }.for_each(|i| bytes[i] = self.data[loc+1]);
    Ok(T::from_le_bytes(Box::new(bytes)))
  }
  pub fn read_instr(&self, loc: usize) -> Result<u32, &str> {
    if loc > self.size { return Err("mem.read_instr() out of bounds") };
    let mut bytes: [u8; 4] = [0;4];
    (0..4).for_each(|i| bytes[i] = self.data[loc+i]);
    Ok(u32::from_le_bytes(bytes))
  }
  pub fn read_signed(&self, loc: usize, s: Size) -> Result<T::Signed, &str> {
    if loc > self.size { return Err("mem.read_signed() out of bounds") };
    use std::{i8, i16, i32};
    let v = match s {
      Size::BYTE => {
        unsafe {
          T::Signed::from(std::mem::transmute::<u8, i8>(self.data[loc]) as i32)
        }
      },
      Size::HALF => {
        let mut bytes : [u8; 2] = [0, 0];
        (0..2).for_each(|i| bytes[i] = self.data[loc+i]);
        T::Signed::from(i16::from_le_bytes(bytes) as i32)
      },
      Size::WORD => {
        let mut bytes : [u8; 4] = [0,0,0,0];
        (0..4).for_each(|i| bytes[i] = self.data[loc+i]);
        T::Signed::from(i32::from_le_bytes(bytes))
      },
      Size::DOUBLE if T::BYTE_SIZE < 8 => return Err("Not sufficient size to read signed double"),
      Size::DOUBLE => {
        let mut bytes : [u8; 8] = Default::default();
        (0..8).for_each(|i| bytes[i] = self.data[loc+i]);
        unimplemented!()
        // TODO create trait for FromLeBytes for signed
        // T::Signed::from(i64::from_le_bytes(bytes))
      },
    };
    Ok(v)
  }
  // queues a write to memory TODO return hit or miss
  // will overwrite things in queue which have same location and memory
  pub fn queue_write(&mut self, loc: usize, data: T, s: Size) -> Result<(), ()> {
    if loc > self.size { return Err(()) };
    self.write_queue.push_back((loc, data, s));
    Ok(())
  }
  pub fn complete_write(&mut self) -> Result<(), &str> {
    match self.write_queue.pop_front() {
      Some((loc, data, sz)) => self.write(loc, data, sz),
      None => Err("No writes remaining"),
    }
  }
  pub fn view(&self, range: Range<usize>) -> MemView<T> {
    assert!(range.start % 4 == 0, "View range start must be word aligned");
    assert!(range.end % 4 == 0, "View range end must be word aligned");
    assert!(range.end < self.size, "View range cannot pass end of memory");
    MemView{ range: range, m: &self, }
  }
}

impl <'a, T : RegData> std::fmt::Display for MemView<'a, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "  ....  ")?;
    for i in self.range.clone().filter(|v| v % 4 == 0) {
      write!(f, "{:08x} ", self.m.read(i, Size::WORD).unwrap())?;
    }
    write!(f, " ....  ")
  }
}

#[test]
fn test_memory_word() {
  let mut mem = create_memory(0x8000usize);
  let data = 0x12345678u32;
  mem.write(0usize, data, Size::WORD).expect("Failed to write memory correctly");
  assert_eq!(mem.read::<u32>(0usize, Size::WORD).unwrap(), data);
}

#[test]
fn test_memory_half() {
  let mut mem = create_memory(0x8000usize);
  let data = 0x12345678u32;
  mem.write(0usize, data, Size::HALF).expect("Failed to write memory correctly");
  let read = mem.read::<u32>(0usize, Size::HALF).unwrap();
  assert_eq!(read, data & 0xffff, "read = 0x{:x}, expected = 0x{:x}", read, data);
}

#[test]
fn test_memory_byte() {
  let mut mem = create_memory(0x8000usize);
  let data = 0x12345678u32;
  mem.write(0usize, data, Size::BYTE).expect("Failed to write memory correctly");
  let read = mem.read::<u32>(0usize, Size::BYTE).unwrap();
  assert_eq!(read, data & 0xff, "read = 0x{:x}, expected = 0x{:x}", read, data);
}

#[test]
fn test_signed_byte() {
  let mut mem = create_memory(0x4usize);
  let data = 0xffu32;
  mem.write(0usize, data, Size::BYTE).expect("Failed to write memory correctly");
  let read = u32::from_signed(mem.read_signed::<u32>(0usize, Size::BYTE).unwrap());
  assert_eq!(read, 0xffffffff);
}

#[test]
fn test_signed_half() {
  let mut mem = create_memory(0x4usize);
  let data = 0xffffu32;
  mem.write(0usize, data, Size::HALF).expect("Failed to write memory correctly");
  let read = u32::from_signed(mem.read_signed::<u32>(0usize, Size::HALF).unwrap());
  assert_eq!(read, 0xffffffff);
}



