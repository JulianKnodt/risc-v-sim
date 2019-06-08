extern crate num;
use num::{Zero, One};
use std::ops::{Add, AddAssign, Sub, Shr, Shl, BitAnd, BitOr, BitXor};

pub trait RegData: Zero + One + Add<Output=Self> + AddAssign + Clone + Copy + From<u32> +
  Sub<Output=Self> + std::fmt::Debug + PartialEq + PartialOrd + Shl<Output=Self> +
  Shr<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self>{

  // Corresponding signed type
  type Signed: Clone + Copy + From<i32> + Add<Output=Self::Signed> + Sub<Output=Self::Signed> +
  PartialEq + PartialOrd + Shl<Output=Self::Signed> + Shr<Output=Self::Signed> +
  BitAnd<Output=Self::Signed>;

  #[inline]
  fn to_signed(self) -> Self::Signed {
    unsafe {
      std::mem::transmute::<Self, Self::Signed>(self)
    }
  }
  #[inline]
  fn from_signed(v: Self::Signed) -> Self {
    unsafe {
      std::mem::transmute::<Self::Signed, Self>(v)
    }
  }
  fn offset(&self, offset: Self::Signed) -> Self;
  fn as_usize(&self) -> usize;
}

impl RegData for u32 {
  type Signed = i32;
  fn offset(&self, s: i32) -> u32 {
    if s < 0 { self - (s.abs() as u32) }
    else { self + (s as u32) }
  }
  fn as_usize(&self) -> usize { *self as usize }
}
impl RegData for u64 {
  type Signed = i64;
  fn offset(&self, s: i64) -> Self {
    if s < 0 { self - (s.abs() as u64) }
    else { self + (s as u64) }
  }
  fn as_usize(&self) -> usize { *self as usize }
}

//pub trait FpRegData: Zero + Add<Output=Self> {}
//impl FpRegData for f32 {}
//impl FpRegData for f64 {}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum RegisterEntry<T : RegData> {
  // most up to date entry
  Valid(T),
  // updated but not written back
  Dirty{ curr: T, new: T },
}

impl <T : RegData> RegisterEntry<T> {
  pub fn is_valid(&self) -> bool {
    match self {
      RegisterEntry::Valid(_) => true,
      RegisterEntry::Dirty{ .. } => false,
    }
  }
  pub fn v(&self) -> T {
    *match self {
      RegisterEntry::Valid(v) => v,
      RegisterEntry::Dirty{ curr: v, .. } => v,
    }
  }
  pub fn invalidate(self, new: T) -> RegisterEntry<T> {
    if let RegisterEntry::Valid(v) = self {
      RegisterEntry::Dirty{ curr: v, new: new }
    } else { self }
  }
}

impl <T : RegData> Default for RegisterEntry<T> {
  fn default() -> Self { RegisterEntry::Valid(T::zero()) }
}

#[derive(PartialEq, Debug)]
pub struct Register<T : RegData> {
  data: Vec<RegisterEntry<T>>,
  pub pc: T,
}

impl <T:RegData>Register<T> {
  pub fn new(num_regs: usize) -> Register<T> {
    Register{
      data: vec![RegisterEntry::default(); num_regs],
      pc: T::zero(),
    }
  }

  pub fn inc_pc(&mut self) {
    self.pc += T::from(crate::mem::WORD_SIZE as u32);
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
