extern crate byteorder;
extern crate sharedbuffer;

use sharedbuffer::SharedReadBuffer;

use std::io::{Seek, Read, SeekFrom, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

pub type Contexts = Vec<(Vec<u8>,Vec<u8>)>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Error(std::io::Error),
    Incomplete(Option<usize>),
}

// extract a value from the byteorder::Result
macro_rules! tryb {
    ($e:expr) => (
        match $e { 
            Ok(r) => r, 
            Err(byteorder::Error::UnexpectedEOF) => {
                return Err(Error::Incomplete(None))
            }
            Err(byteorder::Error::Io(err)) => {
                return Err(Error::Error(err))
            }
        }
    )
}

// extract a value from the io::Result
macro_rules! tryi {
    ($e:expr) => (
        match $e {
            Ok(r) => r,
            Err(err) => return Err(Error::Error(err)),
        }
    )
}

#[derive(PartialEq, Eq, Debug)]
pub struct Tag {
    pub end: bool,
    pub id: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub struct DTable {
    pub entries: Vec<(String,String)>
}

pub struct Message {
    pub tag: Tag,
    pub frame: MessageFrame,
}

pub enum MessageFrame {
    Tdispatch(Tdispatch),
    Rdispatch(Rdispatch),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Tdispatch {
    pub contexts: Contexts,
    pub dest    : String,
    pub dtable  : DTable,
    pub body    : SharedReadBuffer,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Rdispatch {
    pub status  : i8,
    pub contexts: Contexts,
    pub body    : SharedReadBuffer,
}

impl DTable {
    #[inline]
    pub fn new() -> DTable {
        DTable::from(Vec::new())
    }

    #[inline]
    pub fn from(entries: Vec<(String,String)>) -> DTable {
        DTable { entries: entries }
    }

    #[inline]
    pub fn add_entry(&mut self, key: String, value: String) {
        self.entries.push((key,value));
    }
}

impl MessageFrame {
    pub fn frame_size(&self) -> usize {
        match self {
            &MessageFrame::Tdispatch(ref f) => f.frame_size(),
            &MessageFrame::Rdispatch(ref f) => f.frame_size(),
        }
    }

    pub fn frame_id(&self) -> i8 {
        match self {
            &MessageFrame::Tdispatch(_) => 2,
            &MessageFrame::Rdispatch(_) => -2,
        }
    }
}

#[inline]
fn context_size(contexts: &Contexts) -> usize {
    let mut size = 2; // context size

    for &(ref k, ref v) in contexts {
        size += 4; // two lengths
        size += k.len();
        size += v.len();
    }
    size
}

#[inline]
fn dtable_size(table: &DTable) -> usize {
    let mut size = 2; // context size

    for &(ref k, ref v) in &table.entries {
        size += 4; // the two lengths
        size += k.as_bytes().len();
        size += v.as_bytes().len();
    }

    size
}

impl Tdispatch {
    fn frame_size(&self) -> usize {
        let mut size = 2 + // dest size
                       context_size(&self.contexts) +
                       dtable_size(&self.dtable);

        size += self.dest.as_bytes().len();
        size += self.body.remaining();
        size
    }
}

impl Rdispatch {
    fn frame_size(&self) -> usize {
        1 + context_size(&self.contexts) + self.body.remaining()
    }
}

#[inline]
pub fn encode_tag(buffer: &mut Write, tag: &Tag) -> Result<()> {
    let endbit = if tag.end { 1 } else { 0 };
    let bts = [(tag.id >> 16 & 0x7f) as u8 | (endbit << 7), 
               (tag.id >>  8 & 0xff) as u8, 
               (tag.id & 0xff)       as u8];

    tryi!(buffer.write_all(&bts));
    Ok(())
}

pub fn encode_message(buffer: &mut Write, msg: &Message) -> Result<()> {
    // the size is the buffer size + the header (id + tag)
    tryb!(buffer.write_i32::<BigEndian>(msg.frame.frame_size() as i32 + 4));
    tryb!(buffer.write_i8(msg.frame.frame_id()));
    try!(encode_tag(buffer, &msg.tag));
    
    encode_frame(buffer, &msg.frame)
}

fn encode_frame(buffer: &mut Write, frame: &MessageFrame) -> Result<()> {

    match frame {
        &MessageFrame::Tdispatch(ref f) => frames::encode_tdispatch(buffer, f),
        &MessageFrame::Rdispatch(ref f) => frames::encode_rdispatch(buffer, f),
    }
}

pub fn decode_frame(id: i8, buffer: SharedReadBuffer) -> Result<MessageFrame> {
    Ok(match id {
        2  => MessageFrame::Tdispatch(try!(frames::decode_tdispatch(buffer))),
        -2 => MessageFrame::Rdispatch(try!(frames::decode_rdispatch(buffer))),
        _  => panic!("Not implemented")
    })
}

pub fn decode_message_frame(input: &mut SharedReadBuffer) -> Result<Message> {
    if input.remaining() < 8 {
        return Err(Error::Incomplete(None));
    }

    // shoudln't fail, we already ensured the bytes where available
    let size = tryb!(input.read_i32::<BigEndian>());

    if (size as usize) > input.remaining() - 4 {
        tryi!(input.seek(SeekFrom::Current(-4)));
        return Err(Error::Incomplete(None));
    }

    let buff_size = size - 4;

    let tpe = tryb!(input.read_i8());
    let tag = try!(decode_tag(input));

    let msg_buff = tryi!(input.consume_slice(buff_size as usize));

    debug_assert_eq!(msg_buff.remaining(), buff_size as usize);

    let frame = try!(decode_frame(tpe, msg_buff));

    Ok(Message { tag: tag, frame: frame })
}

pub fn decode_tag<T: Read>(r: &mut T) -> Result<Tag> {
    let mut bts = [0; 3];
    let _ = tryi!(r.read(&mut bts));

    let id = (!(1 << 23)) &  // clear the last bit, its for the end flag
            ((bts[0] as u32) << 16 | 
             (bts[1] as u32) <<  8 | 
             (bts[2] as u32));

    Ok(Tag {
        end: (1 << 7) & bts[0] != 0,
        id: id,
    })
}

pub mod frames;

#[cfg(test)]
mod tests;
