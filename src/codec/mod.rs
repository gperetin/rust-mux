extern crate byteorder;

use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};

use std::io;
use std::io::{ErrorKind, Read, Write};
use std::time::Duration;

use super::*;

use std::{u8, u16};

pub mod size;

// concise length checking for encoding length delimited fields
macro_rules! chklen {
    ($e:expr, $len:expr) => ({
        chklen!($e, $len, "Length overflow.")
    });
    ($e:expr, $len:expr, $msg:expr) => {
        if $e.len() > $len as usize {
            return Err(io::Error::new(ErrorKind::InvalidInput, $msg));
        }
    };
}

/// Synchronously read a whole mux `Message`
///
/// This function will synchronously read from the provided `&mut Read` until
/// it has received an entire mux `Message`
///
/// ```rust
/// use std::io::Cursor;
/// use mux::MessageFrame;
/// use mux::codec;
///
/// let mut r = Cursor::new(vec![0,0,0,4,65,0,0,0,1,0,0,0,0]); // ping frame plus 4 0's
/// let frame = codec::read_message(&mut r).unwrap().frame;
/// assert_eq!(frame, MessageFrame::Tping);
/// ```
pub fn read_message<R: Read + ?Sized>(input: &mut R) -> io::Result<Message> {
    let size = {
        let size = try!(input.read_i32::<BigEndian>());
        if size < 4 {
            let msg = format!("Invalid mux frame size: {}. Minimum 4 bytes.", size);
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }

        size as u64
    };

    decode_message(input.take(size))
}

/// Synchronously decode a mux `Message`
///
/// This function will synchronously read from the provided `&mut Read` until
/// it has decoded a message. Message length is assumed to be triggered by an EOF.
/// When using use a continuous stream, such as a `TcpStream`, use `read_message`.
///
/// ```rust
/// use std::io::Cursor;
/// use mux::MessageFrame;
/// use mux::codec;
///
/// let mut r = Cursor::new(vec![65,0,0,0,1]); // ping frame
/// let frame = codec::decode_message(&mut r).unwrap().frame;
/// assert_eq!(frame, MessageFrame::Tping);
/// ```
pub fn decode_message<R: Read>(mut read: R) -> io::Result<Message> {
    let tpe = try!(read.read_i8());
    let tag = try!(decode_tag(&mut read));
    let frame = try!(decode_frame(tpe, &mut read));

    Ok(Message {
        tag: tag,
        frame: frame,
    })
}

/// Synchronously encode a `Message` to the `Write` with the frame size
///
/// Convert the `Message` to a stream of bytes and write it too the `Write`
/// reference. If the `Write` blocks, this function blocks.
///
/// ```rust
/// use std::io::Cursor;
/// use mux::{Message, MessageFrame, Tag};
/// use mux::codec;
///
/// let mut w = Cursor::new(Vec::new());
/// let tag = Tag::new(true, 1);
/// let _ = codec::write_message(&mut w, &Message { tag: tag, frame: MessageFrame::Tping });
/// assert_eq!(w.into_inner(), vec![0,0,0,4,65,0,0,1]);
/// ```
pub fn write_message<W: Write + ?Sized>(buffer: &mut W, msg: &Message) -> io::Result<()> {
    // the size is the buffer size + the header (id + tag)
    try!(buffer.write_i32::<BigEndian>(size::frame_size(&msg.frame) as i32 + 4));
    encode_message(buffer, msg)
}

/// Synchronously encode a `Message` to the `Write`
///
/// Convert the `Message` to a stream of bytes and write it too the `Write`
/// reference. If the `Write` blocks, this function blocks.
///
/// ```rust
/// use std::io::Cursor;
/// use mux::{Message, MessageFrame, Tag};
/// use mux::codec;
///
/// let mut w = Cursor::new(Vec::new());
/// let tag = Tag::new(true, 1);
/// let _ = codec::encode_message(&mut w, &Message { tag: tag, frame: MessageFrame::Tping });
/// assert_eq!(w.into_inner(), vec![65,0,0,1]);
/// ```
pub fn encode_message<W: Write + ?Sized>(buffer: &mut W, msg: &Message) -> io::Result<()> {
    try!(buffer.write_i8(msg.frame.frame_id()));
    try!(encode_tag(buffer, &msg.tag));
    encode_frame(buffer, &msg.frame)
}

// frame codec functions

/// Synchronously encode a `MessageFrame` to the `Write`
///
/// Convert the `FrameMessage` to a stream of bytes and write it too the `Write`
/// reference. If the `Write` blocks, this function blocks.
///
/// ```rust
/// use std::io::Cursor;
/// use mux::{Message, MessageFrame, Tag};
/// use mux::codec;
///
/// let mut w = Cursor::new(Vec::new());
/// let tag = Tag::new(true, 1);
/// let _ = codec::encode_frame(&mut w, &MessageFrame::Tping);
/// assert_eq!(w.into_inner(), vec![]); // Tping is 0 length
/// ```
pub fn encode_frame<W: Write + ?Sized>(writer: &mut W, frame: &MessageFrame) -> io::Result<()> {
    match *frame {
        MessageFrame::Treq(ref f) => encode_treq(writer, f),
        MessageFrame::Rreq(ref f) => encode_rreq(writer, f),
        MessageFrame::Tdispatch(ref f) => encode_tdispatch(writer, f),
        MessageFrame::Rdispatch(ref f) => encode_rdispatch(writer, f),
        MessageFrame::Tinit(ref f) => encode_init(writer, f),
        MessageFrame::Rinit(ref f) => encode_init(writer, f),
        // the following are empty messages
        MessageFrame::Tping => Ok(()),
        MessageFrame::Rping => Ok(()),
        MessageFrame::Tdrain => Ok(()),
        MessageFrame::Rdrain => Ok(()),
        MessageFrame::Tdiscarded(ref msg) => encode_tdiscarded(writer, msg),
        MessageFrame::Tlease(ref d) => encode_tlease_duration(writer, d),
        MessageFrame::Rerr(ref msg) => encode_rerr(writer, msg),
    }
}

/// Synchronously decode a mux `MessageFrame`
///
/// This function will synchronously read from the provided `&mut Read` until
/// it has decoded a frame. Frame length is assumed to be triggered by an EOF.
/// When using use a continuous stream, such as a `TcpStream`, use `Read.take(n)`
/// if you know the frame length or use read_message above to decode an entire
/// `Message` and then extract the frame.
///
/// ```rust
/// use std::io::Cursor;
/// use mux::MessageFrame;
/// use mux::codec;
/// use mux::types;
///
/// let mut r = Cursor::new(vec![]); // ping frame is empty
/// let frame = codec::decode_frame(types::TPING, &mut r).unwrap();
/// assert_eq!(frame, MessageFrame::Tping);
/// ```
pub fn decode_frame<R: Read>(tpe: i8, reader: R) -> io::Result<MessageFrame> {
    Ok(match tpe {
        types::TREQ => MessageFrame::Treq(try!(decode_treq(reader))),
        types::RREQ => MessageFrame::Rreq(try!(decode_rreq(reader))),
        types::TDISPATCH => MessageFrame::Tdispatch(try!(decode_tdispatch(reader))),
        types::RDISPATCH => MessageFrame::Rdispatch(try!(decode_rdispatch(reader))),
        types::TINIT => MessageFrame::Tinit(try!(decode_init(reader))),
        types::RINIT => MessageFrame::Rinit(try!(decode_init(reader))),
        types::TDRAIN => MessageFrame::Tdrain,
        types::RDRAIN => MessageFrame::Rdrain,
        types::TPING => MessageFrame::Tping,
        types::RPING => MessageFrame::Rping,
        types::TLEASE => MessageFrame::Tlease(try!(decode_tlease_duration(reader))),
        types::RERR => MessageFrame::Rerr(try!(decode_rerr(reader))),
        other => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput,
                    format!("Invalid frame type: {}", other))
                );
        }
    })
}


///////////// Tlease codec function

pub fn decode_tlease_duration<R: Read>(mut reader: R) -> io::Result<Duration> {
    let howmuch = try!(reader.read_u8());
    let ticks = try!(reader.read_u64::<BigEndian>());

    if howmuch == 0 {
        Ok(Duration::from_millis(ticks))
    } else {
        let message = format!("Unknown Tlease 'howmuch' code: {}", howmuch);
        Err(io::Error::new(ErrorKind::InvalidData, message))
    }
}

pub fn encode_tlease_duration<W: Write + ?Sized>(writer: &mut W, d: &Duration) -> io::Result<()> {
    let millis = d.as_secs()*1000 + (((d.subsec_nanos() as f64)/1e6) as u64);
    try!(writer.write_u8(0));
    try!(writer.write_u64::<BigEndian>(millis));
    Ok(())
}

///////////// Tdiscarded codecs

#[inline]
pub fn decode_tdiscarded<R: Read>(mut reader: R) -> io::Result<Tdiscarded> {
    let mut bts = [0;3];
    try!(reader.read_exact(&mut bts[..]));
    let id = (bts[0] as u32) << 16 | (bts[1] as u32) <<  8 | (bts[2] as u32);

    Ok(Tdiscarded {
        id: id,
        msg: try!(decode_rerr(reader)), // both just consume the rest of the body as a String
    })
}

#[inline]
pub fn encode_tdiscarded<W: Write + ?Sized>(writer: &mut W, msg: &Tdiscarded) -> io::Result<()> {
    let bts = [
        ((msg.id >> 16) & 0xff) as u8,
        ((msg.id >>  8) & 0xff) as u8,
        ( msg.id        & 0xff) as u8,
    ];
    try!(writer.write_all(&bts[..]));
    writer.write_all(msg.msg.as_bytes())
}

///////////// Tag codec functions

const TAG_END_MASK: u32 = 1 << 23; // 24th bit of tag
const TAG_ID_MASK: u32 = MAX_TAG;

pub fn decode_tag<R: Read + ?Sized>(reader: &mut R) -> io::Result<Tag> {
    let mut bts = [0; 3];
    let _ = try!(reader.read(&mut bts));

    let id = (bts[0] as u32) << 16 |
             (bts[1] as u32) <<  8 |
             (bts[2] as u32);

    Ok(Tag {
        end: id & TAG_END_MASK == 0,
        id: id & TAG_ID_MASK, // clear the last bit, its for the end flag
    })
}

#[inline]
pub fn encode_tag<W: Write + ?Sized>(buffer: &mut W, tag: &Tag) -> io::Result<()> {
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

pub fn encode_headers<W: Write + ?Sized>(writer: &mut W, headers: &Headers) -> io::Result<()> {
    chklen!(headers, u8::MAX, "Header count overflow");
    try!(writer.write_u8(headers.len() as u8));

    for &(ref k, ref v) in headers {
        chklen!(v, u8::MAX, "Header length overflow");

        try!(writer.write_u8(*k));
        try!(writer.write_u8(v.len() as u8));
        try!(writer.write_all(v));
    }
    Ok(())
}

pub fn decode_headers<R: Read + ?Sized>(reader: &mut R) -> io::Result<Headers> {
    let len = try!(reader.read_u8()) as usize;
    let mut acc = Vec::with_capacity(len);

    for _ in 0..len {
        let key = try!(reader.read_u8());
        let val_len = try!(reader.read_u8());
        let mut val = vec![0;val_len as usize];
        try!(reader.read_exact(&mut val[..]));
        acc.push((key, val));
    }

    Ok(acc)
}

///////////// Contexts codec functions

pub fn encode_contexts<W: Write + ?Sized>(writer: &mut W, contexts: &Contexts) -> io::Result<()> {
    chklen!(contexts, u16::MAX, "Context entries overflow");

    try!(writer.write_u16::<BigEndian>(contexts.len() as u16));
    for &(ref k, ref v) in contexts {
        chklen!(k, u16::MAX, "Context key overflow");
        try!(writer.write_u16::<BigEndian>(k.len() as u16));
        try!(writer.write_all(&k[..]));

        chklen!(v, u16::MAX, "Context value overflow");
        try!(writer.write_u16::<BigEndian>(v.len() as u16));
        try!(writer.write_all(&v[..]));
    }

    Ok(())
}

pub fn decode_contexts<R: Read + ?Sized>(reader: &mut R) -> io::Result<Contexts> {
    let len = try!(reader.read_u16::<BigEndian>()) as usize;

    let mut acc = Vec::with_capacity(len);

    for _ in 0..len {
        let key_len = try!(reader.read_u16::<BigEndian>());
        let mut key = vec![0;key_len as usize];
        try!(reader.read_exact(&mut key[..]));

        let val_len = try!(reader.read_u16::<BigEndian>());
        let mut val = vec![0;val_len as usize];
        try!(reader.read_exact(&mut val[..]));
        acc.push((key, val));
    }

    Ok(acc)
}

///////////// Dtab codec functions
pub fn decode_dtab<R: Read + ?Sized>(reader: &mut R) -> io::Result<Dtab> {
    let len = try!(reader.read_u16::<BigEndian>()) as usize;
    let mut acc = Vec::with_capacity(len);

    for _ in 0..len {
        let key_len = try!(reader.read_u16::<BigEndian>());
        let mut key = vec![0;key_len as usize];
        try!(reader.read_exact(&mut key[..]));

        let val_len = try!(reader.read_u16::<BigEndian>());
        let mut val = vec![0;val_len as usize];
        try!(reader.read_exact(&mut val[..]));
        acc.push(Dentry::new(try!(to_string(key)), try!(to_string(val))));
    }

    Ok(Dtab::from_entries(acc))
}

pub fn encode_dtab<W: Write + ?Sized>(writer: &mut W, table: &Dtab) -> io::Result<()> {
    chklen!(table.entries, u16::MAX, "Dtable length overflow");
    try!(writer.write_u16::<BigEndian>(table.entries.len() as u16));

    for dentry in &table.entries {
        // the string encoder will check for overflows
        try!(encode_u16_string(writer, &dentry.key));
        try!(encode_u16_string(writer, &dentry.val));
    }
    Ok(())
}

///////////// Rerr codec functions

#[inline]
pub fn decode_rerr<R: Read>(mut reader: R) -> io::Result<String> {
    let mut data = Vec::new();
    let _ = try!(reader.read_to_end(&mut data));
    to_string(data)
}

#[inline]
pub fn encode_rerr<W: Write + ?Sized>(writer: &mut W, msg: &str) -> io::Result<()> {
    writer.write_all(msg.as_bytes())
}

///////////// Init codec functions

pub fn encode_init<W: Write + ?Sized>(writer: &mut W, msg: &Init) -> io::Result<()> {
    try!(writer.write_u16::<BigEndian>(msg.version));

    // Not going to bother checking for overflow: if a single one of the
    // entries overflows then the entire frame overflows which is not the
    // pervue of this function.

    for &(ref k, ref v) in &msg.headers {
        try!(writer.write_u32::<BigEndian>(k.len() as u32));
        try!(writer.write_all(k));
        try!(writer.write_u32::<BigEndian>(v.len() as u32));
        try!(writer.write_all(v));
    }

    Ok(())
}

pub fn decode_init<R: Read>(mut reader: R) -> io::Result<Init> {
    let mut headers = Vec::new();
    let version = try!(reader.read_u16::<BigEndian>());

    loop {
        let klen = match reader.read_u32::<BigEndian>() {
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                // termination: out of buffer.
                return Ok(
                    Init {
                        version: version,
                        headers: headers,
                    }
                );
            }
            Err(other) => { return Err(other); }
        };

        let mut k = vec![0;klen as usize];
        try!(reader.read_exact(&mut k));

        let vlen = try!(reader.read_u32::<BigEndian>());
        let mut v = vec![0;vlen as usize];
        try!(reader.read_exact(&mut v));

        headers.push((k, v));
    }
}

///////////// Rdispatch codec functions

pub fn encode_rdispatch<W: Write + ?Sized>(writer: &mut W, frame: &Rdispatch) -> io::Result<()> {
    let (status, body) = rmsg_status_body(&frame.msg);

    try!(writer.write_u8(status));
    try!(encode_contexts(writer, &frame.contexts));
    writer.write_all(body)
}

// Expects to consume the whole stream
pub fn decode_rdispatch<R: Read>(mut reader: R) -> io::Result<Rdispatch> {
    let status = try!(reader.read_u8());
    let contexts = try!(decode_contexts(&mut reader));
    let mut body = Vec::new();
    let _ = try!(reader.read_to_end(&mut body));

    Ok(Rdispatch {
        contexts: contexts,
        msg: try!(decode_rmsg_body(status, body)),
    })
}

///////////// Rreq codec functions

pub fn encode_rreq<W: Write + ?Sized>(writer: &mut W, frame: &Rmsg) -> io::Result<()> {
    let (status, body) = rmsg_status_body(frame);
    try!(writer.write_u8(status));
    writer.write_all(body)
}

pub fn decode_rreq<R: Read>(mut reader: R) -> io::Result<Rmsg> {
    let status = try!(reader.read_u8());
    let mut body = Vec::new();
    try!(reader.read_to_end(&mut body));
    decode_rmsg_body(status, body)
}

///////////// Tdispatch codec functions

pub fn decode_tdispatch<R: Read>(mut reader: R) -> io::Result<Tdispatch> {
    let contexts = try!(decode_contexts(&mut reader));
    let dest = try!(decode_u16_string(&mut reader));
    let dtab = try!(decode_dtab(&mut reader));

    let mut body = Vec::new();
    let _ = try!(reader.read_to_end(&mut body));

    Ok(Tdispatch {
        contexts: contexts,
        dest: dest,
        dtab: dtab,
        body: body,
    })
}

pub fn encode_tdispatch<W: Write + ?Sized>(writer: &mut W, msg: &Tdispatch) -> io::Result<()> {
    try!(encode_contexts(writer, &msg.contexts));
    try!(encode_u16_string(writer, &msg.dest));
    try!(encode_dtab(writer, &msg.dtab));
    writer.write_all(&msg.body)
}

///////////// Treq codec functions

pub fn decode_treq<R: Read>(mut reader: R) -> io::Result<Treq> {
    let headers = try!(decode_headers(&mut reader));
    let mut body = Vec::new();

    let _ = try!(reader.read_to_end(&mut body));
    Ok(Treq {
        headers: headers,
        body: body,
    })
}

#[inline]
pub fn encode_treq<W: Write + ?Sized>(writer: &mut W, msg: &Treq) -> io::Result<()> {
    try!(encode_headers(writer, &msg.headers));
    writer.write_all(&msg.body)
}

#[inline]
fn rmsg_status_body(msg: &Rmsg) -> (u8, &[u8]) {
    match *msg {
        Rmsg::Ok(ref body) => (0, body.as_ref()),
        Rmsg::Error(ref msg) => (1, msg.as_bytes()),
        Rmsg::Nack(ref msg) => (2, msg.as_bytes()),
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
pub fn decode_u16_string<R: Read + ?Sized>(reader: &mut R) -> io::Result<String> {
    let str_len = try!(reader.read_u16::<BigEndian>());
    let mut s = vec![0; str_len as usize];

    try!(reader.read_exact(&mut s));

    to_string(s)
}

#[inline]
pub fn encode_u16_string<W: Write + ?Sized>(writer: &mut W, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();

    chklen!(bytes, u16::MAX, "16bit length delimited string overflow");
    try!(writer.write_u16::<BigEndian>(bytes.len() as u16));
    writer.write_all(bytes)
}

#[inline]
fn to_string(vec: Vec<u8>) -> io::Result<String> {
    match String::from_utf8(vec) {
        Ok(s) => Ok(s),
        Err(_) => Err(io::Error::new(ErrorKind::InvalidData, "Invalid UTF8 field")),
    }
}
