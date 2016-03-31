extern crate byteorder;

use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};

use std::io;
use std::io::{Cursor, ErrorKind, Read, Write};
use std::time::Duration;

use super::*;

use std::{u8, u16};

pub mod size;

// extract a value from the byteorder::Result
#[macro_export]
macro_rules! tryb {
    ($e:expr) => (
        match $e {
            Ok(r) => r,
            Err(byteorder::Error::UnexpectedEOF) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "End of input"
                ));
            }
            Err(byteorder::Error::Io(err)) => {
                return Err(err);
            }
        }
    )
}

// write the Message to the Write
pub fn encode_message(buffer: &mut Write, msg: &Message) -> io::Result<()> {
    // the size is the buffer size + the header (id + tag)
    tryb!(buffer.write_i32::<BigEndian>(size::frame_size(&msg.frame) as i32 + 4));
    tryb!(buffer.write_i8(msg.frame.frame_id()));
    try!(encode_tag(buffer, &msg.tag));

    encode_frame(buffer, &msg.frame)
}

pub fn encode_frame(buffer: &mut Write, frame: &MessageFrame) -> io::Result<()> {
    match frame {
        &MessageFrame::Treq(ref f) => encode_treq(buffer, f),
        &MessageFrame::Rreq(ref f) => encode_rreq(buffer, f),
        &MessageFrame::Tdispatch(ref f) => encode_tdispatch(buffer, f),
        &MessageFrame::Rdispatch(ref f) => encode_rdispatch(buffer, f),
        &MessageFrame::Tinit(ref f) => encode_init(buffer, f),
        &MessageFrame::Rinit(ref f) => encode_init(buffer, f),
        // the following are empty messages
        &MessageFrame::Tping => Ok(()),
        &MessageFrame::Rping => Ok(()),
        &MessageFrame::Tdrain => Ok(()),
        &MessageFrame::Rdrain => Ok(()),
        &MessageFrame::Tlease(ref d) => {
            let millis = d.as_secs()*1000 + (((d.subsec_nanos() as f64)/1e6) as u64);
            tryb!(buffer.write_u8(0));
            tryb!(buffer.write_u64::<BigEndian>(millis));
            Ok(())
        }
        &MessageFrame::Rerr(ref msg) => {
            buffer.write_all(msg.as_bytes())
        }
    }
}

// Decodes `data` into a frame if type `tpe`
pub fn decode_frame(tpe: i8, data: &[u8]) -> io::Result<MessageFrame> {
    Ok(match tpe {
        types::TREQ => MessageFrame::Treq(try!(decode_treq(data))),
        types::RREQ => MessageFrame::Rreq(try!(decode_rreq(data))),
        types::TDISPATCH => MessageFrame::Tdispatch(try!(decode_tdispatch(data))),
        types::RDISPATCH => MessageFrame::Rdispatch(try!(decode_rdispatch(data))),
        types::TINIT => MessageFrame::Tinit(try!(decode_init(data))),
        types::RINIT => MessageFrame::Rinit(try!(decode_init(data))),
        types::TDRAIN => MessageFrame::Tdrain,
        types::RDRAIN => MessageFrame::Rdrain,
        types::TPING => MessageFrame::Tping,
        types::RPING => MessageFrame::Rping,
        types::TLEASE => {
            let mut buffer = Cursor::new(data);
            let _ = try!(buffer.read_u8());
            let ticks = try!(buffer.read_u64::<BigEndian>());
            MessageFrame::Tlease(Duration::from_millis(ticks))
        }
        types::RERR => MessageFrame::Rerr(try!(decode_rerr(data))),
        other => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput,
                    format!("Invalid frame type: {}", other))
                );
        }
    })
}

// Read an entire frame buffer
pub fn read_frame(input: &mut Read) -> io::Result<MuxPacket> {
    let size = {
        let size = tryb!(input.read_i32::<BigEndian>());
        if size < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, "Invalid mux frame size"
            ));
        }
        size as usize
    };

    let tpe = tryb!(input.read_i8());
    let tag = try!(decode_tag(input));

    let mut buf = vec![0;size-4];
    try!(input.read_exact(&mut buf));
    Ok(MuxPacket {
        tpe: tpe,
        tag: tag,
        buffer: buf,
    })
}

// This is a synchronous function that will read a whole message from the `Read`
pub fn read_message(input: &mut Read) -> io::Result<Message> {
    let packet = try!(read_frame(input));
    decode_message(&packet)
}

// expects a SharedReadBuffer of the whole mux frame
pub fn decode_message(input: &MuxPacket) -> io::Result<Message> {
    let frame = try!(decode_frame(input.tpe, &input.buffer));
    Ok(Message {
        tag: input.tag.clone(),
        frame: frame,
    })
}

///////////// Tag codec functions

const TAG_END_MASK: u32 = 1 << 23; // 24th bit of tag
const TAG_ID_MASK: u32 = !TAG_END_MASK;

pub fn decode_tag<R: Read + ?Sized>(buffer: &mut R) -> io::Result<Tag> {
    let mut bts = [0; 3];
    let _ = try!(buffer.read(&mut bts));

    let id = (bts[0] as u32) << 16 |
             (bts[1] as u32) <<  8 |
             (bts[2] as u32);

    Ok(Tag {
        end: id & TAG_END_MASK == 0,
        id: id & TAG_ID_MASK, // clear the last bit, its for the end flag
    })
}

#[inline]
pub fn encode_tag(buffer: &mut Write, tag: &Tag) -> io::Result<()> {
    let bytes = {
        let id = tag.id;
        let endbit = if tag.end { 0 } else { 1 };
        [(id >> 16 & 0x7f) as u8 | (endbit << 7),
         (id >> 8 & 0xff) as u8,
         (id >> 0 & 0xff) as u8]
    };

    buffer.write_all(&bytes)
}

///////////// headers codec functions

pub fn encode_headers(buffer: &mut Write, headers: &Headers) -> io::Result<()> {
    if headers.len() > u8::MAX as usize {
        return Err(io::Error::new(
            ErrorKind::InvalidInput, format!("Too many headers: {}", headers.len())
        ));
    }

    try!(buffer.write_u8(headers.len() as u8));

    for &(ref k, ref v) in headers {
        if v.len() > u8::MAX as usize {
            return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid header size"));
        }

        try!(buffer.write_u8(*k));
        try!(buffer.write_u8(v.len() as u8));
        try!(buffer.write_all(v));
    }
    Ok(())
}

pub fn decode_headers<R: Read + ?Sized>(buffer: &mut R) -> io::Result<Headers> {
    let len = tryb!(buffer.read_u8()) as usize;
    let mut acc = Vec::with_capacity(len);

    for _ in 0..len {
        let key = tryb!(buffer.read_u8());
        let val_len = tryb!(buffer.read_u8());
        let mut val = vec![0;val_len as usize];
        try!(buffer.read_exact(&mut val[..]));
        acc.push((key, val));
    }

    Ok(acc)
}

///////////// Contexts codec functions

pub fn encode_contexts(buffer: &mut Write, contexts: &Contexts) -> io::Result<()> {
    if contexts.len() > u16::MAX as usize {
        return Err(io::Error::new(ErrorKind::InvalidInput, "Too many contexts to encode"));

    }

    tryb!(buffer.write_u16::<BigEndian>(contexts.len() as u16));

    for &(ref k, ref v) in contexts {
        if k.len() > u16::MAX as usize || v.len() > u16::MAX as usize {
            return Err(io::Error::new(ErrorKind::InvalidInput, "Context too large to encode"));
        }

        tryb!(buffer.write_u16::<BigEndian>(k.len() as u16));
        try!(buffer.write_all(&k[..]));

        tryb!(buffer.write_u16::<BigEndian>(v.len() as u16));
        try!(buffer.write_all(&v[..]));
    }

    Ok(())
}

pub fn decode_contexts<R: Read + ?Sized>(buffer: &mut R) -> io::Result<Contexts> {
    let len = tryb!(buffer.read_u16::<BigEndian>()) as usize;

    let mut acc = Vec::with_capacity(len);

    for _ in 0..len {
        let key_len = tryb!(buffer.read_u16::<BigEndian>());
        let mut key = vec![0;key_len as usize];
        try!(buffer.read_exact(&mut key[..]));

        let val_len = tryb!(buffer.read_u16::<BigEndian>());
        let mut val = vec![0;val_len as usize];
        try!(buffer.read_exact(&mut val[..]));
        acc.push((key, val));
    }

    Ok(acc)
}

///////////// Dtab codec functions

pub fn decode_dtab<R: Read + ?Sized>(buffer: &mut R) -> io::Result<Dtab> {
    let ctxs: Vec<(Vec<u8>, Vec<u8>)> = try!(decode_contexts(buffer));
    let mut acc = Vec::with_capacity(ctxs.len());

    for (k, v) in ctxs {
        let k = try!(to_string(k));
        let v = try!(to_string(v));
        acc.push((k, v));
    }

    Ok(Dtab { entries: acc })
}

pub fn encode_dtab<R: Write + ?Sized>(buffer: &mut R, table: &Dtab) -> io::Result<()> {
    tryb!(buffer.write_u16::<BigEndian>(table.entries.len() as u16));

    for &(ref k, ref v) in &table.entries {
        try!(encode_u16_string(buffer, k));
        try!(encode_u16_string(buffer, v));
    }
    Ok(())
}

///////////// Rerr codec functions

#[inline]
pub fn decode_rerr<R: Read>(mut buffer: R) -> io::Result<String> {
    let mut data = Vec::new();
    let _ = try!(buffer.read_to_end(&mut data));
    to_string(data)
}

///////////// Init codec functions

pub fn encode_init(buffer: &mut Write, msg: &Init) -> io::Result<()> {
    tryb!(buffer.write_u16::<BigEndian>(msg.version));

    for &(ref k, ref v) in &msg.headers {
        tryb!(buffer.write_u32::<BigEndian>(k.len() as u32));
        try!(buffer.write_all(k));
        tryb!(buffer.write_u32::<BigEndian>(v.len() as u32));
        try!(buffer.write_all(v));
    }

    Ok(())
}

pub fn decode_init(data: &[u8]) -> io::Result<Init> {
    let datalen = data.len() as u64;
    let mut buffer = Cursor::new(data);

    let version = tryb!(buffer.read_u16::<BigEndian>());
    let mut headers = Vec::new();

    while buffer.position() < datalen {
        let klen = tryb!(buffer.read_u32::<BigEndian>());
        let mut k = vec![0;klen as usize];
        try!(buffer.read_exact(&mut k));

        let vlen = tryb!(buffer.read_u32::<BigEndian>());
        let mut v = vec![0;vlen as usize];
        try!(buffer.read_exact(&mut v));

        headers.push((k, v));
    }

    Ok(Init {
        version: version,
        headers: headers,
    })
}

///////////// Rdispatch codec functions

pub fn encode_rdispatch(buffer: &mut Write, frame: &Rdispatch) -> io::Result<()> {
    let (status, body) = rmsg_status_body(&frame.msg);

    tryb!(buffer.write_u8(status));
    try!(encode_contexts(buffer, &frame.contexts));
    buffer.write_all(body)
}

// Expects to consume the whole stream
pub fn decode_rdispatch<R: Read>(mut buffer: R) -> io::Result<Rdispatch> {
    let status = tryb!(buffer.read_u8());
    let contexts = try!(decode_contexts(&mut buffer));
    let mut body = Vec::new();
    let _ = try!(buffer.read_to_end(&mut body));

    Ok(Rdispatch {
        contexts: contexts,
        msg: try!(decode_rmsg_body(status, body)),
    })
}

///////////// Rreq codec functions

pub fn encode_rreq(buffer: &mut Write, frame: &Rmsg) -> io::Result<()> {
    let (status, body) = rmsg_status_body(frame);
    tryb!(buffer.write_u8(status));
    buffer.write_all(body)
}

pub fn decode_rreq<R: Read>(mut buffer: R) -> io::Result<Rmsg> {
    let status = try!(buffer.read_u8());
    let mut body = Vec::new();
    try!(buffer.read_to_end(&mut body));
    decode_rmsg_body(status, body)
}

///////////// Tdispatch codec functions

pub fn decode_tdispatch<R: Read>(mut buffer: R) -> io::Result<Tdispatch> {
    let contexts = try!(decode_contexts(&mut buffer));
    let dest = try!(decode_u16_string(&mut buffer));
    let dtab = try!(decode_dtab(&mut buffer));

    let mut body = Vec::new();
    let _ = try!(buffer.read_to_end(&mut body));

    Ok(Tdispatch {
        contexts: contexts,
        dest: dest,
        dtab: dtab,
        body: body,
    })
}

pub fn encode_tdispatch(buffer: &mut Write, msg: &Tdispatch) -> io::Result<()> {
    try!(encode_contexts(buffer, &msg.contexts));
    try!(encode_u16_string(buffer, &msg.dest));
    try!(encode_dtab(buffer, &msg.dtab));
    buffer.write_all(&msg.body)
}

///////////// Treq codec functions

pub fn decode_treq<R: Read>(mut buffer: R) -> io::Result<Treq> {
    let headers = try!(decode_headers(&mut buffer));
    let mut body = Vec::new();

    let _ = try!(buffer.read_to_end(&mut body));
    Ok(Treq {
        headers: headers,
        body: body,
    })
}

#[inline]
pub fn encode_treq(buffer: &mut Write, msg: &Treq) -> io::Result<()> {
    try!(encode_headers(buffer, &msg.headers));
    buffer.write_all(&msg.body)
}

#[inline]
fn rmsg_status_body(msg: &Rmsg) -> (u8, &[u8]) {
    match msg {
        &Rmsg::Ok(ref body) => (0, body.as_ref()),
        &Rmsg::Error(ref msg) => (1, msg.as_bytes()),
        &Rmsg::Nack(ref msg) => (2, msg.as_bytes()),
    }
}

#[inline]
fn decode_rmsg_body(status: u8, body: Vec<u8>) -> io::Result<Rmsg> {
    match status {
        0 => Ok(Rmsg::Ok(body)),
        1 => Ok(Rmsg::Error(try!(to_string(body)))),
        2 => Ok(Rmsg::Nack(try!(to_string(body)))),
        other => Err(
            io::Error::new(ErrorKind::InvalidData, format!("Invalid status code: {}", other))
        )
    }
}

// tools for operating on Strings

// decode a utf8 string with length specified by a u16 prefix byte
#[inline]
pub fn decode_u16_string<R: Read + ?Sized>(buffer: &mut R) -> io::Result<String> {
    let str_len = tryb!(buffer.read_u16::<BigEndian>());
    let mut s = vec![0; str_len as usize];

    try!(buffer.read_exact(&mut s));

    to_string(s)
}

#[inline]
pub fn encode_u16_string<W: Write + ?Sized>(buffer: &mut W, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    assert!(bytes.len() <= u16::MAX as usize);
    tryb!(buffer.write_u16::<BigEndian>(bytes.len() as u16));
    buffer.write_all(bytes)
}

#[inline]
fn to_string(vec: Vec<u8>) -> io::Result<String> {
    match String::from_utf8(vec) {
        Ok(s) => Ok(s),
        Err(_) => Err(io::Error::new(ErrorKind::InvalidData, "Invalid UTF8 field")),
    }
}
