use std::fmt::{Display, Formatter, Result};

use bimap::BiMap;
use once_cell::sync::Lazy;
use wardstone_core::primitive::hash::*;

pub static HASH_REPR: Lazy<BiMap<Hash, &str>> = Lazy::new(|| {
  let mut m = BiMap::new();
  m.insert(SHA256, "sha256");
  m
});

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HashFunc {
  pub func: Hash,
  name: String,
}

impl Display for HashFunc {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    let HashFunc { name, .. } = self;
    write!(f, "{name}")
  }
}

impl From<Hash> for HashFunc {
  fn from(func: Hash) -> Self {
    let name = HASH_REPR.get_by_left(&func).unwrap_or(&"UNRECOGNISED");
    Self {
      func,
      name: name.to_string(),
    }
  }
}

impl From<&str> for HashFunc {
  fn from(name: &str) -> Self {
    let func = *HASH_REPR.get_by_right(&name).unwrap_or(&HASH_NOT_SUPPORTED);
    HashFunc {
      func,
      name: name.to_string(),
    }
  }
}
