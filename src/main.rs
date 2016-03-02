extern crate sharedbuffer;
extern crate byteorder;
extern crate mux;

use sharedbuffer::SharedReadBuffer;

use std::net::TcpStream;
use std::io::{Read, Write};

fn test_trequest<S: Read+Write>(socket: &mut S) {

    let v = "Hello, world!".to_string().into_bytes();
    let b = SharedReadBuffer::new(v);

    let frame = mux::Tdispatch::basic("/foo".to_string(), b);
    let msg = mux::Message::end(1, frame);

    let mut buf = Vec::new();
    mux::encode_message(&mut buf, &msg).unwrap();
    socket.write_all(&buf).unwrap();

    let msg = mux::read_message(socket).unwrap();

    match &msg.frame {
        &mux::MessageFrame::Rdispatch(ref msg) => {
            let s = std::str::from_utf8(&msg.body).unwrap();
            println!("Response: {}", s);
        }
        other => {
            panic!(format!("Recieved unexpected frame: {:?}", other));
        }
    }
}

fn test_ping<S: Read+Write>(socket: &mut S) {
    let msg = mux::Message::end(2, mux::MessageFrame::TPing);
    let mut buf = Vec::new();
    mux::encode_message(&mut buf, &msg).unwrap();

    socket.write_all(&buf).unwrap();

    let msg = mux::read_message(socket).unwrap();
    match &msg.frame {
        &mux::MessageFrame::RPing => {
            println!("Ping successful!");
        }
        other => {
            panic!(format!("Received unexpected frame: {:?}", other));
        }
    }
}

fn main() {

  let mut socket = TcpStream::connect(("localhost", 9000)).unwrap();
  println!("Testing ping frame.");
  test_ping(&mut socket);

  println!("Testing TRequest frame.");
  test_trequest(&mut socket);
}
