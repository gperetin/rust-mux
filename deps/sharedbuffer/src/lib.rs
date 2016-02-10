#![crate_type = "lib"]
#![crate_name = "sharedbuffer"]

use std::sync::Arc;

pub struct SharedBuffer {
  inner: Arc<Vec<u8>>
}

pub struct SharedReadBuffer {
  inner: Arc<Vec<u8>>,
  offset: usize,
  limit: usize
}

impl SharedBuffer {
  pub fn new(size: usize) -> SharedBuffer {
    SharedBuffer {
      inner: Arc::new(Vec::with_capacity(size))
    }
  }

  pub fn to_readbuffer(&self) -> SharedReadBuffer {
    SharedReadBuffer {
      inner: self.inner.clone(),
      offset: 0,
      limit: self.inner.len()      
    }
  }
}

#[test]
fn it_works() {
}

