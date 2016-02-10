#[macro_use]
extern crate nom;

use nom::{IResult, Needed, be_u16, be_u32, rest};
use nom::IResult::*;
use nom::Err::*;


pub struct MessageFrame<'a> {
  size: u32,
  tpe:   i8,
  tag:  u32,
  bytes: &'a [u8]
}

pub enum Message {
  Tdispatch(Tdispatch)
}

pub struct Tdispatch {
  contexts: Vec<(Vec<u8>, Vec<u8>)>,
  dst:      String,
  dtab:     Vec<(Vec<u8>, Vec<u8>)>,
  req:      Vec<u8>
}

named!(context<&[u8], (Vec<u8>, Vec<u8>)>,
  chain!(
    cnta: be_u16      ~
    btsa: take!(cnta) ~
    cntb: be_u16      ~
    btsb: take!(cntb),
    || { (btsa.to_vec(),btsb.to_vec()) }
  )
); 

named!(contexts<&[u8], Vec<(Vec<u8>, Vec<u8>)> >,
  chain!(
    count: be_u16 ~
    ctxs: many_m_n!(count as usize, count as usize, context),
    ||{ ctxs }
  )
);


named!(decode_Tdispatch <&[u8], Message>,
  chain!(
    cts: contexts    ~
    dstsz: be_u16    ~
    dest: map_res!(take!(dstsz), |bts: &[u8]|{ String::from_utf8(bts.to_vec()) } ) ~
    dtabz: contexts ~ /* this is a stop-gap! */
    bytes: rest,
    || {
      Message::Tdispatch(Tdispatch {
	  contexts: cts,
	  dst: dest,
	  dtab: dtabz,
	  req: bytes.to_vec()
      })
    }
  )
);

named!(message_frame<&[u8], MessageFrame>,
  chain!(
    size: be_u32 ~
    meta: be_u32 ~
    bytes: take!(size-4),
    || {
      let tpe = ((meta & 0xFF000000) >> 24) as i8;
      let tag = meta & 0x00FFFFFF;
      MessageFrame{ size: size, tpe: tpe, tag: tag, bytes: bytes }
    }
  )
);

fn decode_message(input: &[u8]) -> IResult<&[u8], Message> {
  match message_frame(input) {
    Done(i, frame) => {
      match frame.tpe {
        2 => decode_Tdispatch(i),
        _ => panic!("Not implemented")
      }
    }
    Error(e)      => Error(e),
    Incomplete(e) => Incomplete(e)
  }
}

#[test]
fn it_works() {
  let a = Some(4);
  if let Some(4) = a {
    println!("It is the number 4!");
  }
}

