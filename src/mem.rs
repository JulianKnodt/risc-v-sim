use std::u32;

pub const WORD_SIZE: usize = 4;
pub enum Size {
  WORD,
  HALF,
  BYTE,
}

pub struct Memory {
  data: Vec<u8>,
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




