use std::u32;

pub const WORD_SIZE: usize = 4;
pub enum Size {
  WORD,
  HALF,
  BYTE,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Memory {
  pub data: Vec<u8>,
  size: usize,
}


pub fn create_memory(size: usize) -> Memory {
  Memory { data: vec![0; size], size: size, }
}

impl Memory {
  pub fn write(&mut self, loc: usize, data: u32, s: Size) -> Result<(), ()> {
    if loc > self.size { return Err(()) };
    let bytes = data.to_le_bytes();
    match s {
      Size::BYTE => {
        assert_ne!(bytes[0], 0);
        self.data[loc] = bytes[0];
      },
      Size::HALF => {
        (0..2).for_each(|i| self.data[loc+i] = bytes[i]);
      },
      Size::WORD => {
        (0..4).for_each(|i| self.data[loc+i] = bytes[i]);
      },
    };
    Ok(())
  }
  pub fn read(&self, loc: usize, s: Size) -> Result<u32, ()> {
    if loc > self.size { return Err(()) };
    match s {
      Size::BYTE => {
        Ok(self.data[loc] as u32)
      },
      Size::HALF => {
        let mut bytes : [u8; 4] = [0,0,0,0];
        (0..2).for_each(|i| bytes[i] = self.data[loc+i]);
        Ok(u32::from_le_bytes(bytes))
      },
      Size::WORD => {
        let mut bytes : [u8; 4] = [0,0,0,0];
        (0..4).for_each(|i| bytes[i] = self.data[loc+i]);
        Ok(u32::from_le_bytes(bytes))
      },
    }
  }
  pub fn read_signed(&self, loc: usize, s: Size) -> Result<u32, ()> {
    if loc > self.size { return Err(()) };
    use std::{i8, i16, i32};
    use std::mem::transmute;
    let v = match s {
      Size::BYTE => {
        unsafe {
          transmute::<u8, i8>(self.data[loc]) as i32
        }
      },
      Size::HALF => {
        let mut bytes : [u8; 2] = [0, 0];
        (0..2).for_each(|i| bytes[i] = self.data[loc+i]);
        i16::from_le_bytes(bytes) as i32
      },
      Size::WORD => {
        let mut bytes : [u8; 4] = [0,0,0,0];
        (0..4).for_each(|i| bytes[i] = self.data[loc+i]);
        i32::from_le_bytes(bytes)
      },
    };
    unsafe { Ok(transmute::<i32, u32>(v)) }
  }
}

#[test]
fn test_memory_word() {
  let mut mem = create_memory(0x8000usize);
  let data = 0x12345678u32;
  mem.write(0usize, data, Size::WORD).expect("Failed to write memory correctly");
  assert_eq!(mem.read(0usize, Size::WORD).unwrap(), data);
}

#[test]
fn test_memory_half() {
  let mut mem = create_memory(0x8000usize);
  let data = 0x12345678u32;
  mem.write(0usize, data, Size::HALF).expect("Failed to write memory correctly");
  let read = mem.read(0usize, Size::HALF).unwrap();
  assert_eq!(read, data & 0xffff, "read = 0x{:x}, expected = 0x{:x}", read, data);
}

#[test]
fn test_memory_byte() {
  let mut mem = create_memory(0x8000usize);
  let data = 0x12345678u32;
  mem.write(0usize, data, Size::BYTE).expect("Failed to write memory correctly");
  let read = mem.read(0usize, Size::BYTE).unwrap();
  assert_eq!(read, data & 0xff, "read = 0x{:x}, expected = 0x{:x}", read, data);
}

#[test]
fn test_signed_byte() {
  let mut mem = create_memory(0x4usize);
  let data = 0xff;
  mem.write(0usize, data, Size::BYTE).expect("Failed to write memory correctly");
  let read = mem.read_signed(0usize, Size::BYTE).unwrap();
  assert_eq!(read, 0xffffffff);
}

#[test]
fn test_signed_half() {
  let mut mem = create_memory(0x4usize);
  let data = 0xffff;
  mem.write(0usize, data, Size::HALF).expect("Failed to write memory correctly");
  let read = mem.read_signed(0usize, Size::HALF).unwrap();
  assert_eq!(read, 0xffffffff);
}



