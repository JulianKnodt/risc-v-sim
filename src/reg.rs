extern crate num;
use num::{Zero, One};
use std::ops::{Add, AddAssign, Sub, Shr, Shl, BitAnd, BitOr, BitXor};
use std::mem::transmute;
use crate::fixed_size_deque::FixedSizeDeque;

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

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct RegisterEntry<T : RegData>(FixedSizeDeque<T>);

impl <T : RegData> RegisterEntry<T> {
  pub fn as_usize(&self) -> usize { self.v().as_usize() }
  pub fn v(&self) -> T { self.0.back().unwrap() }
  pub fn write(&mut self, new: T) { assert!(self.0.push_back(new)) }
  pub fn writeback(&mut self) { self.0.pop_front(); }
  pub fn assign(&mut self, v: T) {
    self.0.clear();
    self.0.push_front(v);
  }
  // reset on error to oldest value still in pipeline
  pub fn reset(&mut self) { self.0.truncate(1) }
}

impl <T : RegData> Default for RegisterEntry<T> {
  fn default() -> Self {
    let mut empty = FixedSizeDeque::new();
    empty.push_back(T::zero());
    RegisterEntry(empty)
  }
}

#[derive(PartialEq, Debug)]
pub struct Register<T : RegData> {
  pub(crate) data: Vec<RegisterEntry<T>>,
  pub pc: RegisterEntry<T>,
}

impl <T:RegData>Register<T> {
  pub fn new(num_regs: usize) -> Register<T> {
    Register{
      data: vec![RegisterEntry::default(); num_regs],
      pc: RegisterEntry::default(),
    }
  }

  pub fn inc_pc(&mut self) {
    // self.pc.0.iter_mut().for_each(|v| *v += T::from(crate::mem::WORD_SIZE as u32));
  }
}

use std::ops::{Index, IndexMut};
impl <T : RegData> Index<u32> for Register<T> {
  type Output = RegisterEntry<T>;
  fn index(&self, i: u32) -> &RegisterEntry<T> {
    &self.data[i as usize]
  }
}

impl <T : RegData> IndexMut<u32> for Register<T> {
  fn index_mut(&mut self, i: u32) -> &mut RegisterEntry<T> {
    &mut self.data[i as usize]
  }
}
