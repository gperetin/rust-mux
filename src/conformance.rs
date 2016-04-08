
use std::fmt::Debug;
use std::io;
use std::io::Cursor;
use std::time::Duration;

use super::*;
use codec::*;
use codec::size::*;

const BUFFER_STR: &'static str = "hello world";

fn writer() -> Cursor<Vec<u8>> {
    Cursor::new(Vec::new())
}

fn body() -> Vec<u8> {
    BUFFER_STR.to_owned().into_bytes()
}

fn check<T, F1, F2, F3>(buf: Vec<u8>, decode: F1, expected: T, encode: F2, size: F3) -> ()
    where F1: Fn(Cursor<Vec<u8>>) -> io::Result<T>,
          F2: Fn(&mut Cursor<Vec<u8>>, &T) -> io::Result<()>,
          F3: FnOnce(T) -> MessageFrame,
          T: PartialEq + Debug
{
    let msg = decode(Cursor::new(buf.clone())).unwrap();
    assert_eq!(msg, expected);

    let mut w = writer();
    encode(&mut w, &expected).unwrap();

    assert_eq!(frame_size(&size(expected)), buf.len());
    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_treq() {
    // Request type: Treq(1,None,BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
               0x6f, 0x72, 0x6c, 0x64, ];

    let expected = Treq { headers: Vec::new(), body: body() };
    check(buf, &decode_treq, expected, &encode_treq, |t| { MessageFrame::Treq(t)});
}

#[test]
fn test_rreqok() {
    // Request type: RreqOk(1,BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
               0x6f, 0x72, 0x6c, 0x64, ];

    let expected = Rmsg::Ok(body());
    check(buf, &decode_rreq, expected, &encode_rreq, |t| { MessageFrame::Rreq(t) });
}

#[test]
fn test_rreqerror() {
    // Request type: RreqError(1,hello world)
    let buf = vec![0x01, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
               0x6f, 0x72, 0x6c, 0x64, ];

    let expected = Rmsg::Error(BUFFER_STR.to_owned());
    check(buf, &decode_rreq, expected, &encode_rreq, |t| MessageFrame::Rreq(t));
}

#[test]
fn test_rreqnack() {
    // Request type: RreqNack(1)
    let buf = vec![0x02, ];

    let expected = Rmsg::Nack("".to_owned());
    check(buf, &decode_rreq, expected, &encode_rreq, |t| { MessageFrame::Rreq(t) });
}

#[test]
fn test_tdispatch_1() {
    // Request type: Tdispatch(1,List(),Path(path),Dtab(),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x00, 0x00, 0x05, 0x2f, 0x70, 0x61, 0x74,
               0x68, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
               0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, ];

    let expected = Tdispatch {
        contexts: Vec::new(),
        dest: "/path".to_owned(),
        dtab: Dtab::new(),
        body: body(),
    };

    check(buf, &decode_tdispatch, expected, &encode_tdispatch, |t| MessageFrame::Tdispatch(t) );
}

#[test]
fn test_tdispatch_2() {
    // Request type: Tdispatch(1,List((BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))),Path(path),Dtab(),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x01, 0x00, 0x0b, 0x68, 0x65, 0x6c, 0x6c,
               0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00,
               0x0b, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
               0x6f, 0x72, 0x6c, 0x64, 0x00, 0x05, 0x2f, 0x70,
               0x61, 0x74, 0x68, 0x00, 0x00, 0x68, 0x65, 0x6c,
               0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
               ];

    let msg = decode_tdispatch(buf.as_slice()).unwrap();
    let expected = Tdispatch {
        contexts: vec![(body(), body())],
        dest: "/path".to_owned(),
        dtab: Dtab::new(),
        body: body(),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_tdispatch(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
    check(buf, &decode_tdispatch, expected, &encode_tdispatch, |t| MessageFrame::Tdispatch(t) );
}

#[test]
fn test_tdispatch_3() {
    // Request type: Tdispatch(1,List(),Path(path),Dtab(/f/foo=>/go),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x00, 0x00, 0x05, 0x2f, 0x70, 0x61, 0x74,
               0x68, 0x00, 0x01, 0x00, 0x06, 0x2f, 0x66, 0x2f,
               0x66, 0x6f, 0x6f, 0x00, 0x03, 0x2f, 0x67, 0x6f,
               0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f,
               0x72, 0x6c, 0x64, ];

    let dtab = Dtab::from_entries(
        vec![Dentry::new("/f/foo".to_owned(), "/go".to_owned())]
    );

    let expected = Tdispatch {
        contexts: vec![],
        dest: "/path".to_owned(),
        dtab: dtab,
        body: body(),
    };

    check(buf, &decode_tdispatch, expected, &encode_tdispatch, |t| MessageFrame::Tdispatch(t) );
}

#[test]
fn test_rdispatchok_1() {
    // Request type: RdispatchOk(1,List(),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
               0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, ];

    let expected = Rdispatch {
        contexts: vec![],
        msg: Rmsg::Ok(body()),
    };

    check(buf, &decode_rdispatch, expected, &encode_rdispatch, |t| MessageFrame::Rdispatch(t) );
}

#[test]
fn test_rdispatchok_2() {
    // Request type: RdispatchOk(1,List((BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x00, 0x01, 0x00, 0x0b, 0x68, 0x65, 0x6c,
               0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
               0x00, 0x0b, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20,
               0x77, 0x6f, 0x72, 0x6c, 0x64, 0x68, 0x65, 0x6c,
               0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
               ];

    let expected = Rdispatch {
        contexts: vec![(body(), body())],
        msg: Rmsg::Ok(body()),
    };

    check(buf, &decode_rdispatch, expected, &encode_rdispatch, |t| MessageFrame::Rdispatch(t) );
}

#[test]
fn test_rdispatcherror() {
    // Request type: RdispatchError(1,List(),hello world)
    let buf = vec![0x01, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
               0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, ];

    let expected = Rdispatch {
        contexts: vec![],
        msg: Rmsg::Error(BUFFER_STR.to_owned()),
    };

    check(buf, &decode_rdispatch, expected, &encode_rdispatch, |t| MessageFrame::Rdispatch(t) );
}

#[test]
fn test_rdispatchnack() {
    // Request type: RdispatchNack(1,List())
    let buf = vec![0x02, 0x00, 0x00, ];

    let expected = Rdispatch {
        contexts: vec![],
        msg: Rmsg::Nack("".to_owned()),
    };

    check(buf, &decode_rdispatch, expected, &encode_rdispatch, |t| MessageFrame::Rdispatch(t) );
}

#[test]
fn test_rerr() {
    // Request type: Rerr(1,hello world)
    let buf = vec![0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f,
               0x72, 0x6c, 0x64, ];

    let msg = decode_rerr(buf.as_slice()).unwrap();
    let expected = BUFFER_STR.to_owned();

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rerr(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_tdiscarded() {
    // Request type: Tdiscarded(1,hello world)
    let buf = vec![0x00, 0x00, 0x01, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
               0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, ];

    let expected = Tdiscarded {
        id: 1,
        msg: BUFFER_STR.to_owned(),
    };

    check(buf, &decode_tdiscarded, expected, &encode_tdiscarded, |t| MessageFrame::Tdiscarded(t));
}

#[test]
fn test_tlease() {
    // Request type: Tlease(0,1000)
    let buf = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, ];

    let expected = Duration::from_millis(1000);

    check(buf, &decode_tlease_duration, expected, &encode_tlease_duration, |t| MessageFrame::Tlease(t));
}
