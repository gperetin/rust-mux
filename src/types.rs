//! Wire identification tag of the mux message types.
//!
//! The encoding of these tags is two's compliment one-byte integer
//! where positive integers are T-messages and their negative compliment
//! are the coresponding R-messages. T-messages greater than 63 are
//! consider session control messages along with their R-message compliment
//! while all other messages are consider application messages.

pub const TREQ: i8 = 1;
pub const RREQ: i8 = -1;

pub const TDISPATCH: i8 = 2;
pub const RDISPATCH: i8 = -2;

pub const TINIT: i8 = 68;
pub const RINIT: i8 = -68;

pub const TDRAIN: i8 = 64;
pub const RDRAIN: i8 = -64;

pub const TPING: i8 = 65;
pub const RPING: i8 = -65;

pub const TDISCARDED: i8 = 66;
pub const TLEASE: i8 = 67;

pub const RERR: i8 = -128;
