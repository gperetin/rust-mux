extern crate byteorder;

mod dtab;
pub mod codec;
pub mod session;
pub mod types;

pub use dtab::*;
use std::time::Duration;

/// Headers for a `Treq`.
pub type Headers = Vec<(u8, Vec<u8>)>;

/// Contexts of dispatch and init messages.
pub type Contexts = Vec<(Vec<u8>, Vec<u8>)>;

/// Maximum value of a mux Tag
pub const MAX_TAG: u32 = (1 << 23) - 1;

/// Id number and end flag for message frames.
///
/// Every message has a `Tag` following the frame length and frame
/// type on the wire. The frame id is limited to 23 bits of precision
/// while bit 24 signals if the message stream is ending. This only
/// applies to the `Tdispatch` and `Rdispatch` frames.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Tag {
    pub end: bool,
    pub id: u32,
}

/// An entire mux packet.
///
/// The `Message` type contains enough information to encode an
/// entire packet.
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub tag: Tag,
    pub frame: MessageFrame,
}

/// Wrapper of the various mux packet types.
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
    Tdiscarded(Tdiscarded),
    Tlease(Tlease),
    Rerr(Rerr),
}

// Structs that model the message frame types of the mux protocol

/// Representation of the mux `Treq` types.
#[derive(PartialEq, Eq, Debug)]
pub struct Treq {
    pub headers: Headers,
    pub body: Vec<u8>,
}

/// Representation of a mux `Rreq` and `Rdispatch` message body.
#[derive(PartialEq, Eq, Debug)]
pub enum Rmsg {
    /// Successful response.
    Ok(Vec<u8>),
    /// Response failed. The `String` describes the error.
    Error(String),
    /// Negative acknowledgment. The `String` describes the reason.
    Nack(String),
}

/// Representation of a mux `Tdispatch` frame.
#[derive(PartialEq, Eq, Debug)]
pub struct Tdispatch {
    pub contexts: Contexts,
    pub dest: String,
    pub dtab: Dtab,
    pub body: Vec<u8>,
}

/// Representation of a mux `Rdispatch` frame.
#[derive(PartialEq, Eq, Debug)]
pub struct Rdispatch {
    pub contexts: Contexts,
    pub msg: Rmsg,
}

/// Representation of a mux `Tinit` and `Rinit` frame.
#[derive(PartialEq, Eq, Debug)]
pub struct Init {
    pub version: u16,
    pub headers: Contexts,
}

/// Representation of a mux `Tdiscarded` frame.
#[derive(PartialEq, Eq, Debug)]
pub struct Tdiscarded {
    pub id: u32,
    pub msg: String,
}

/// Representation of a mux `Tlease` frame.
#[derive(PartialEq, Eq, Debug)]
pub struct Tlease {
    pub duration: Duration,
}

/// Representation of a mux `Rerr` frame.
#[derive(PartialEq, Eq, Debug)]
pub struct Rerr {
    pub msg: String,
}

impl Tag {
    #[inline]
    /// Construct a new `Tag`.
    pub fn new(end: bool, id: u32) -> Tag {
        assert!(id <= MAX_TAG);
        Tag { end: end, id: id, }
    }
}

impl MessageFrame {
    /// Get the `i8` value coresponding the a `MessageFrame`.
    pub fn frame_id(&self) -> i8 {
        match *self {
            MessageFrame::Treq(_) => types::TREQ,
            MessageFrame::Rreq(_) => types::RREQ,
            MessageFrame::Tdispatch(_) => types::TDISPATCH,
            MessageFrame::Rdispatch(_) => types::RDISPATCH,
            MessageFrame::Tinit(_) => types::TINIT,
            MessageFrame::Rinit(_) => types::RINIT,
            MessageFrame::Tdrain => types::TDRAIN,
            MessageFrame::Rdrain => types::RDRAIN,
            MessageFrame::Tping => types::TPING,
            MessageFrame::Rping => types::RPING,
            MessageFrame::Tdiscarded(_) => types::TDISCARDED,
            MessageFrame::Tlease(_) => types::TLEASE,
            MessageFrame::Rerr(_) => types::RERR,
        }
    }
}

impl Tdispatch {
    /// Construct a new `Tdispatch` frame with the provided destination and body.
    pub fn new(dest: String, body: Vec<u8>) -> Tdispatch {
        Tdispatch {
            contexts: Vec::new(),
            dest: dest,
            dtab: Dtab::new(),
            body: body,
        }
    }
}


#[cfg(test)]
mod tests;

#[cfg(test)]
mod conformance;
