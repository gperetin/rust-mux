extern crate byteorder;

use std::time::Duration;

pub mod types;
pub mod codec;

pub type Headers = Vec<(u8, Vec<u8>)>;
pub type Contexts = Vec<(Vec<u8>, Vec<u8>)>;

pub const MAX_TAG: u32 = (1 << 23) - 1;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Tag {
    pub end: bool,
    pub id: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Dtab {
    pub entries: Vec<(String, String)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub tag: Tag,
    pub frame: MessageFrame,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MessageFrame {
    Treq(Treq),
    Rreq(Rmsg),
    Tdispatch(Tdispatch),
    Rdispatch(Rdispatch),
    Tinit(Init),
    Rinit(Init),
    Tdrain,
    Rdrain,
    Tping,
    Rping,
    Rerr(String),
    Tlease(Duration),   // Notification of a lease of resources for the specified duration
    // Tdiscarded(String), // Sent by a client to alert the server of a discarded message
}

#[derive(PartialEq, Eq, Debug)]
pub struct Treq {
    pub headers: Headers,
    pub body: Vec<u8>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Tdispatch {
    pub contexts: Contexts,
    pub dest: String,
    pub dtab: Dtab,
    pub body: Vec<u8>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Rdispatch {
    pub contexts: Contexts,
    pub msg: Rmsg,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Rmsg {
    Ok(Vec<u8>),
    Error(String),
    Nack(String),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Init {
    pub version: u16,
    pub headers: Contexts,
}

impl Message {
    #[inline]
    pub fn end(id: u32, frame: MessageFrame) -> Message {
        Message {
            tag: Tag::new(true, id),
            frame: frame,
        }
    }
}

impl Tag {
    #[inline]
    pub fn new(end: bool, id: u32) -> Tag {
        assert!(id <= MAX_TAG);
        Tag { end: end, id: id, }
    }
}

impl Dtab {
    #[inline]
    pub fn new() -> Dtab {
        Dtab::from(Vec::new())
    }

    #[inline]
    pub fn from(entries: Vec<(String, String)>) -> Dtab {
        Dtab { entries: entries }
    }

    #[inline]
    pub fn add_entry(&mut self, key: String, value: String) {
        self.entries.push((key, value));
    }
}

impl MessageFrame {
    pub fn frame_id(&self) -> i8 {
        match self {
            &MessageFrame::Treq(_) => types::TREQ,
            &MessageFrame::Rreq(_) => types::RREQ,
            &MessageFrame::Tdispatch(_) => types::TDISPATCH,
            &MessageFrame::Rdispatch(_) => types::RDISPATCH,
            &MessageFrame::Tinit(_) => types::TINIT,
            &MessageFrame::Rinit(_) => types::RINIT,
            &MessageFrame::Tdrain => types::TDRAIN,
            &MessageFrame::Rdrain => types::RDRAIN,
            &MessageFrame::Tping => types::TPING,
            &MessageFrame::Rping => types::RPING,
            &MessageFrame::Tlease(_) => types::TLEASE,
            &MessageFrame::Rerr(_) => types::RERR,
        }
    }
}

impl Tdispatch {
    pub fn basic_(dest: String, body: Vec<u8>) -> Tdispatch {
        Tdispatch {
            contexts: Vec::new(),
            dest: dest,
            dtab: Dtab::new(),
            body: body,
        }
    }

    pub fn basic(dest: String, body: Vec<u8>) -> MessageFrame {
        MessageFrame::Tdispatch(Tdispatch::basic_(dest, body))
    }
}


#[cfg(test)]
mod tests;
