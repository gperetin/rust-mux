

extern crate sharedbuffer;
extern crate byteorder;

use byteorder::{ReadBytesExt, BigEndian};

use std::io::{Error,ErrorKind,Read};

use sharedbuffer::SharedReadBuffer;
use super::*;

fn decode_contexts<R: Read>(buffer: &mut R) -> DecodeResult<Vec<(Vec<u8>,Vec<u8>)>> {
    let len = tryb!(buffer.read_u16::<BigEndian>());

    let mut acc = Vec::new();

    for _ in 0..len {
        let key_len = tryb!(buffer.read_u16::<BigEndian>());
        let mut key = vec![0;key_len as usize];
        tryi!(buffer.read_exact(&mut key[..]));

        let val_len = tryb!(buffer.read_u16::<BigEndian>());
        let mut val = vec![0;val_len as usize];
        tryi!(buffer.read_exact(&mut val[..]));
        acc.push((key,val));
    }

    Ok(acc)
}

fn decode_dtable<R: Read>(buffer: &mut R) -> DecodeResult<DTable> {
    let ctxs : Vec<(Vec<u8>,Vec<u8>)> = try!(decode_contexts(buffer));
    
    let mut acc = Vec::with_capacity(ctxs.len());

    for (k,v) in ctxs {
        let k = try!(to_string(k));
        let v = try!(to_string(v));
        acc.push((k,v));
    }

    Ok(DTable{ entries: acc })
}

fn to_string(vec: Vec<u8>) -> DecodeResult<String> {
    match String::from_utf8(vec) {
        Ok(s) => Ok(s),
        Err(_) => Err(DecodeFail::Error(
                      Error::new(ErrorKind::Other, "Failed to decode UTF8")
                      )),
    }
}

// decode a utf8 string with length specified by a u16 prefix byte
fn decode_string<R: Read>(buffer: &mut R) -> DecodeResult<String> {
    let str_len = tryb!(buffer.read_u16::<BigEndian>());
    let mut s = vec![0; str_len as usize];

    tryi!(buffer.read_exact(&mut s));

    to_string(s)
}

pub fn decode_tdispatch(mut buffer: SharedReadBuffer) -> DecodeResult<MessageFrame> {
    let contexts = try!(decode_contexts(&mut buffer));

    let dest = try!(decode_string(&mut buffer));
    let dtable = try!(decode_dtable(&mut buffer));
    let body = buffer.consume_remaining();

    Ok(MessageFrame::Tdispatch {
        contexts: contexts,
        dest: dest,
        dtable: dtable,
        body: body,
    })
}

