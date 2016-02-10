
extern crate sharedbuffer;

use sharedbuffer::SharedBuffer;

fn main() {
  println!("Hello, world!");
  let b = SharedBuffer::new(10);
  let c = b.to_readbuffer();
  
}

