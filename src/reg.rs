extern crate num;
use std::ops::{Shr, Shl, BitAnd, BitOr, BitXor, Index};
use std::mem::transmute;
use std::collections::VecDeque;
use std::hash::Hash;
use std::fmt::{Display, Debug, LowerHex};

// TODO

pub trait RegData: num::Unsigned + Clone + Copy + From<u32> + From<u8> + Ord + Shl<Output=Self>
  + Shr<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self>
  + LowerHex + Debug + Display + Hash {

  // Corresponding signed type
  type Signed: num::Signed + From<i32> + Ord + Shl<Output=Self::Signed>
    + Shr<Output=Self::Signed> + BitAnd<Output=Self::Signed> + Display;

  fn to_signed(self) -> Self::Signed;
  fn from_signed(v: Self::Signed) -> Self;
  fn offset(&self, offset: Self::Signed) -> Self;
  fn as_usize(&self) -> usize;

  const BYTE_SIZE: usize = std::mem::size_of::<Self>();
  // Byte Representation of Data
  // The best version would be this, but doesn't compile yet
  // Some bugs on github opened for it
  // fn to_le_bytes(&self) -> [u8; Self::BYTE_SIZE];
  fn to_le_bytes(&self) -> Box<[u8]>;
  fn from_le_bytes(bytes: Box<[u8]>) -> Self;
}

impl RegData for u32 {
  type Signed = i32;
  fn offset(&self, s: i32) -> u32 {
    if s < 0 { self - (s.abs() as u32) } else { self + (s as u32) }
  }
  fn as_usize(&self) -> usize { *self as usize }
  #[inline]
  fn to_signed(self) -> Self::Signed { unsafe { transmute::<Self, Self::Signed>(self) } }
  #[inline]
  fn from_signed(v: Self::Signed) -> Self { unsafe { transmute::<Self::Signed, Self>(v) } }

  fn to_le_bytes(&self) -> Box<[u8]> { Box::new(u32::to_le_bytes(*self)) }
  fn from_le_bytes(bytes: Box<[u8]>) -> Self {
    let mut temp: [u8; Self::BYTE_SIZE] = Default::default();
    temp.copy_from_slice(&bytes);
    Self::from_le_bytes(temp)
  }
}
impl RegData for u64 {
  type Signed = i64;
  fn offset(&self, s: i64) -> Self {
    if s < 0 { self - (s.abs() as u64) }
    else { self + (s as u64) }
  }
  fn as_usize(&self) -> usize { *self as usize }
  #[inline]
  fn to_signed(self) -> Self::Signed { unsafe { transmute::<Self, Self::Signed>(self) } }
  #[inline]
  fn from_signed(v: Self::Signed) -> Self { unsafe { transmute::<Self::Signed, Self>(v) } }

  fn to_le_bytes(&self) -> Box<[u8]> { Box::new(u64::to_le_bytes(*self)) }
  fn from_le_bytes(bytes: Box<[u8]>) -> Self {
    let mut temp: [u8; Self::BYTE_SIZE] = Default::default();
    temp.copy_from_slice(&bytes);
    Self::from_le_bytes(temp)
  }
}

#[derive(PartialEq, Debug)]
pub struct Register<T : RegData> {
  data: Vec<T>,
  unwritten: VecDeque<(usize, T)>,
  pc: T,
}

impl <T : RegData> std::fmt::Display for Register<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    for i in 0..self.data.len() {
      writeln!(f, "[ x{:02}: {:08x} | {} ]", i, self.data[i], self.data[i].to_signed())?;
    }
    writeln!(f, "[ pc : {:08x} ]", self.pc())
  }
}

impl <T:RegData>Register<T> {
  pub fn new(num_regs: usize) -> Register<T> {
    Register{
      data: vec![T::zero(); num_regs],
      unwritten: VecDeque::new(),
      pc: T::zero(),
    }
  }

  pub fn pc(&self) -> T { self.pc }
  pub fn inc_pc(&mut self) {
    self.pc = self.pc + T::from(crate::mem::WORD_SIZE as u32);
  }
  pub fn force_assign(&mut self, rd: u32, v: T) { self.data[rd as usize] = v }
  pub fn assign(&mut self, rd: u32, v: T) {
    if rd == 0 { self.data[0] = T::zero() }
    else { self.unwritten.push_back((rd as usize, v)) }
  }
  pub fn writeback(&mut self, rd: u32) -> bool {
    if rd == 0 { return true }
    else if self.unwritten.len() == 0 { return false };
    let (rd, v) = self.unwritten.pop_front().expect("Failed writeback");
    self.data[rd] = v;
    true
  }
  pub fn assign_pc(&mut self, v: T) { self.pc = v }
}

impl <T: RegData>Index<u32> for Register<T> {
  type Output = T;
  fn index(&self, i: u32) -> &T {
    let i = i as usize;
    self.unwritten.iter().find(|&v| v.0 == i).map(|v| &v.1).unwrap_or(&self.data[i])
  }
}
