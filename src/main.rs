
extern crate sharedbuffer;
extern crate byteorder;

use sharedbuffer::SharedReadBuffer;
use byteorder::{BigEndian, WriteBytesExt};

fn main() {
  println!("Hello, world!");

  let mut v = Vec::new();

  for i in 0..10 {
    v.write_u64::<BigEndian>(i).unwrap();
  }

  let b = SharedReadBuffer::new(v);

  let c = b.slice(8);

  println!("Hello world!");
  
}

