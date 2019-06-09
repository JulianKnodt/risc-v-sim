use crate::reg::RegData;
pub const WORD_SIZE: usize = 4;
pub enum Size {
  WORD,
  HALF,
  BYTE,
  // DOUBLE,
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
  pub fn write<T : RegData>(&mut self, loc: usize, data: T, s: Size) -> Result<(), ()> {
    if loc > self.size { return Err(()) };
    let bytes = data.to_le_bytes();
    match s {
      Size::BYTE => self.data[loc] = bytes[0],
      Size::HALF => (0..2).for_each(|i| self.data[loc+i] = bytes[i]),
      Size::WORD => (0..4).for_each(|i| self.data[loc+i] = bytes[i]),
    };
    Ok(())
  }
  pub fn read<T : RegData>(&self, loc: usize, s: Size) -> Result<T, ()> {
    if loc > self.size { return Err(()) };
    match s {
      Size::BYTE => Ok(T::from(self.data[loc])),
      Size::HALF => {
        let mut bytes : [u8; 4] = [0,0,0,0];
        (0..2).for_each(|i| bytes[i] = self.data[loc+i]);
        Ok(T::from_le_bytes(Box::new(bytes)))
      },
      Size::WORD => {
        let mut bytes : [u8; 4] = [0,0,0,0];
        (0..4).for_each(|i| bytes[i] = self.data[loc+i]);
        Ok(T::from_le_bytes(Box::new(bytes)))
      },
    }
  }
  pub fn read_signed<T : RegData>(&self, loc: usize, s: Size) -> Result<T::Signed, ()> {
    if loc > self.size { return Err(()) };
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
    };
    Ok(v)
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



