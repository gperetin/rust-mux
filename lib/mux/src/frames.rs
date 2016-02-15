

extern crate sharedbuffer;
extern crate byteorder;

use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};

use std::io;
use std::io::{ErrorKind, Read, Write};

use sharedbuffer::SharedReadBuffer;
use super::*;

use std::u16;

fn encode_contexts<W: Write>(buffer: &mut W, contexts: &Contexts) -> Result<()> {
    // TODO: these shouldn't be asserts.
    assert!(contexts.len() <= u16::MAX as usize);
    tryb!(buffer.write_u16::<BigEndian>(contexts.len() as u16));
    
    for &(ref k, ref v) in contexts {
        assert!(k.len() <= u16::MAX as usize);
        assert!(v.len() <= u16::MAX as usize);

        tryb!(buffer.write_u16::<BigEndian>(k.len() as u16));
        tryi!(buffer.write_all(&k[..]));

        tryb!(buffer.write_u16::<BigEndian>(v.len() as u16));
        tryi!(buffer.write_all(&v[..]));
    }

    Ok(())
}

fn decode_contexts<R: Read>(buffer: &mut R) -> Result<Contexts> {
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

fn decode_dtable<R: Read>(buffer: &mut R) -> Result<DTable> {
    let ctxs : Vec<(Vec<u8>,Vec<u8>)> = try!(decode_contexts(buffer));
    
    let mut acc = Vec::with_capacity(ctxs.len());

    for (k,v) in ctxs {
        let k = try!(to_string(k));
        let v = try!(to_string(v));
        acc.push((k,v));
    }

    Ok(DTable{ entries: acc })
}

fn to_string(vec: Vec<u8>) -> Result<String> {
    match String::from_utf8(vec) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::Error(
                      io::Error::new(ErrorKind::Other, "Failed to decode UTF8")
                      )),
    }
}

// decode a utf8 string with length specified by a u16 prefix byte
fn decode_string<R: Read>(buffer: &mut R) -> Result<String> {
    let str_len = tryb!(buffer.read_u16::<BigEndian>());
    let mut s = vec![0; str_len as usize];

    tryi!(buffer.read_exact(&mut s));

    to_string(s)
}

pub fn decode_tdispatch(mut buffer: SharedReadBuffer) -> Result<MessageFrame> {
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

