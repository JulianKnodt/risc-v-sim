// TODO make this into a macro so it can have various sizes
// arbitrary fixed size deque so it can implement copy
const FIXED_SIZE : usize = 5;
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
pub struct FixedSizeDeque<T: Copy + Default> {
  data: [T; FIXED_SIZE],
  head: usize,
  tail: usize,
  at_cap: bool,
}

impl <T: Copy + Default>FixedSizeDeque<T> {
  pub fn new() -> FixedSizeDeque<T> { Default::default() }
  pub fn len(&self) -> usize {
    if self.head > self.tail { self.head - self.tail }
    else if self.head < self.tail { self.head + FIXED_SIZE - self.tail }
    else if self.at_cap { self.capacity() }
    else { 0 }
  }
  pub fn back(&self) -> Option<T> {
    if self.len() == 0 { None } else { Some(self.data[self.tail-1]) }
  }
  pub fn capacity(&self) -> usize { FIXED_SIZE }
  pub fn push_back(&mut self, t: T) -> bool {
    if self.at_cap { return false };
    self.data[self.tail] = t;
    self.tail = (self.tail+1) % FIXED_SIZE;
    self.at_cap = self.head == self.tail;
    true
  }
  pub fn pop_front(&mut self) -> Option<T> {
    if self.len() == 0 { return None }
    let out = Some(self.data[self.head]);
    self.head = (self.head+1) % FIXED_SIZE;
    self.at_cap = self.head == self.tail;
    out
  }
  pub fn truncate(&mut self, len: usize) {
    if len > self.capacity() { return }
    self.tail = (self.head + len) % FIXED_SIZE;
  }
  pub fn clear(&mut self) { self.truncate(0) }
  pub fn push_front(&mut self, t: T) -> bool {
    if self.at_cap { return false };
    self.head = if self.head == 0 { FIXED_SIZE - 1 } else { self.head - 1 };
    self.data[self.head] = t;
    true
  }
}

#[cfg(test)]
mod test_fsd {
}
