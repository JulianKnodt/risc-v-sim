extern crate num;
use num::{Zero, One};
use std::ops::{Add, AddAssign, Sub, Shr, Shl, BitAnd, BitOr, BitXor, Index};
use std::mem::transmute;
use std::collections::VecDeque;

pub trait RegData: Zero + One + Add<Output=Self> + AddAssign + Clone + Copy + Default + From<u32>
  + From<u8> + Sub<Output=Self> + std::fmt::Debug + PartialEq + PartialOrd + Shl<Output=Self>
  + Shr<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self>
  + std::fmt::LowerHex {

  // Corresponding signed type
  type Signed: Clone + Copy + From<i32> + Add<Output=Self::Signed>
    + Sub<Output=Self::Signed> + PartialEq + PartialOrd + Shl<Output=Self::Signed>
    + Shr<Output=Self::Signed> + BitAnd<Output=Self::Signed>;

  fn to_signed(self) -> Self::Signed;
  fn from_signed(v: Self::Signed) -> Self;
  fn offset(&self, offset: Self::Signed) -> Self;
  fn as_usize(&self) -> usize;

  // Byte Representation of Data
  // The best version would be this, but doesn't compile yet
  // Some bugs on github opened for it
  // const BYTE_SIZE: usize;
  // fn to_le_bytes(&self) -> [u8; Self::BYTE_SIZE];
  const BYTE_SIZE: usize;
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

  const BYTE_SIZE: usize = 4;
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

  const BYTE_SIZE: usize = 8;
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
  pc: VecDeque<T>,
}

impl <T:RegData>Register<T> {
  pub fn new(num_regs: usize) -> Register<T> {
    let mut pc = VecDeque::new();
    pc.push_front(T::zero());
    Register{
      data: vec![T::zero(); num_regs],
      unwritten: VecDeque::new(),
      pc: pc,
    }
  }

  pub fn pc(&self) -> T { *self.pc.back().unwrap() }
  pub fn inc_pc(&mut self) {
    let v = self.pc.back_mut().unwrap();
    *v = *v + T::from(crate::mem::WORD_SIZE as u32);
  }
  pub fn assign(&mut self, rd: u32, v: T) {
    if rd == 0 { self.data[0] = T::zero() }
    else { self.unwritten.push_back((rd as usize, v)) }
  }
  pub fn writeback(&mut self) -> bool {
    if self.unwritten.len() == 1 { return false };
    let (rd, v) = self.unwritten.pop_front().unwrap();
    self.data[rd] = v;
    true
  }
  pub fn assign_pc(&mut self, v: T) { self.pc.push_back(v) }
  pub fn pc_writeback(&mut self) -> bool {
    if self.pc.len() == 1 { return false }
    self.pc.pop_front();
    true
  }
}

impl <T: RegData>Index<u32> for Register<T> {
  type Output = T;
  fn index(&self, i: u32) -> &T {
    let i = i as usize;
    self.unwritten.iter().find(|&v| v.0 == i).map(|v| &v.1).unwrap_or(&self.data[i])
  }
}
