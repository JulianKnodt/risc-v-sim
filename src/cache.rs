// are u32s sufficient to store in cache?

// todo implement Caches
pub struct CacheRow {
  data: Vec<u32>,
  tag: u32,
  valid: bool,
  dirty: bool,
}

enum Associativity {
  N(u32),
  Full,
}

pub struct Cache {
  assoc: Associativity,
  data: Vec<CacheRow>,
}
