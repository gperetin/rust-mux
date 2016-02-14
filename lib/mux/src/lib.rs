extern crate byteorder;
extern crate sharedbuffer;

use sharedbuffer::SharedReadBuffer;

use std::io::{Seek, Read, SeekFrom};

use byteorder::{ReadBytesExt, BigEndian};

pub type DecodeResult<T> = Result<T, DecodeFail>;

// extract a value from the byteorder::Result
macro_rules! tryb {
    ($e:expr) => (
        match $e { 
            Ok(r) => r, 
            Err(byteorder::Error::UnexpectedEOF) => {
                return Err(DecodeFail::Incomplete(None))
            }
            Err(byteorder::Error::Io(err)) => {
                return Err(DecodeFail::Error(err))
            }
        }
    )
}

// extract a value from the io::Result
macro_rules! tryi {
    ($e:expr) => (
        match $e {
            Ok(r) => r,
            Err(err) => return Err(DecodeFail::Error(err)),
        }
    )
}

#[derive(Debug)]
pub enum DecodeFail {
    Error(std::io::Error),
    Incomplete(Option<usize>),
}

pub struct Tag {
    pub end: bool,
    pub id: u32,
}

pub struct DTable {
    pub entries: Vec<(String,String)>
}

fn read_tag<T: Read>(r: &mut T) -> byteorder::Result<Tag> {
    let mut bts = [0; 3];
    let _ = try!(r.read(&mut bts));

    let id = (!(1 << 23)) &  // clear the last bit
            ((bts[0] as u32) << 16 | 
             (bts[1] as u32) <<  8 | 
             (bts[2] as u32));

    Ok(Tag {
        end: (1 << 7) & bts[0] != 0,
        id: id,
    })
}

pub struct Message {
    pub tag: Tag,
    pub frame: MessageFrame,
}

pub enum MessageFrame {
    Tdispatch {
        contexts: Vec<(Vec<u8>, Vec<u8>)>,
        dest    : String,
        dtable  : DTable,
        body    : SharedReadBuffer,
    },
}

pub fn decode_frame(id: i8, tag: Tag, buffer: SharedReadBuffer) -> DecodeResult<Message> {
    let frame = try!(match id {
        2 => frames::decode_tdispatch(buffer),
        _ => panic!("Not implemented")
    });

    Ok(Message { tag: tag, frame: frame })
}

pub fn decode_message_frame(input: &mut SharedReadBuffer) -> DecodeResult<Message> {
    if input.remaining() < 8 {
        return Err(DecodeFail::Incomplete(None));
    }

    // shoudln't fail, we already ensured the bytes where available
    let size = tryb!(input.read_i32::<BigEndian>());

    if (size as usize) > input.remaining() - 4 {
        tryi!(input.seek(SeekFrom::Current(-4)));
        return Err(DecodeFail::Incomplete(None));
    }

    let buff_size = size - 4;

    let tpe = tryb!(input.read_i8());
    let tag = tryb!(read_tag(input));

    let msg_buff = tryi!(input.consume_slice(buff_size as usize));

    debug_assert_eq!(msg_buff.remaining(), buff_size as usize);

    decode_frame(tpe, tag, msg_buff)
}

#[test]
fn it_works() {
    let a = Some(4);
    if let Some(4) = a {
        println!("It is the number 4!");
    }
}

mod frames;
