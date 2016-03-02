
extern crate sharedbuffer;
extern crate byteorder;
extern crate mux;

use sharedbuffer::SharedReadBuffer;

use byteorder::{ReadBytesExt, BigEndian, ByteOrder};
use std::net::TcpStream;
use std::io::{Read, Write};

fn send_request(buffer: &[u8]) {
    let mut socket = TcpStream::connect(("localhost", 9000)).unwrap();
    socket.write_all(buffer).unwrap();

    let mut buf = vec![0; 4];
    socket.read_exact(&mut buf).unwrap();
    let frame_size = BigEndian::read_i32(&buf) as usize;

    println!("Frame size: {}", frame_size);
    buf.resize(frame_size+4,0);

    socket.read_exact(&mut buf[4..]).unwrap();

    println!("Read buffer: {:?}", &buf);

    let mut buf = SharedReadBuffer::new(buf);
    let msg = mux::decode_message(&mut buf).unwrap();

    match &msg.frame {
        &mux::MessageFrame::Rdispatch(ref msg) => {
            let s = std::str::from_utf8(&msg.body).unwrap();
            println!("Response: {}", s);
        }
        other => {
            panic!(format!("Oh no: {:?}", other));
        }
    }

    // println!("Message: {:?}", &msg);
}

fn main() {
  println!("Hello, world!");

  let v = "Hello, world!".to_string().into_bytes();
  let b = SharedReadBuffer::new(v);

  let frame = mux::Tdispatch::basic("/foo".to_string(), b);
  let msg = mux::Message::end(1, frame);

  let mut buf = Vec::new();
  mux::encode_message(&mut buf, &msg).unwrap();

  send_request(&buf);
}

