
use std::io::Cursor;
use std::time::Duration;

use super::*;
use codec::*;

const BUFFER_STR: &'static str = "hello world";

fn writer() -> Cursor<Vec<u8>> {
    Cursor::new(Vec::new())
}

fn body() -> Vec<u8> {
    BUFFER_STR.to_owned().into_bytes()
}

#[test]
fn test_treq() {
    // Request type: Treq(1,None,BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
               0x6f, 0x72, 0x6c, 0x64, ];

    let msg = decode_treq(buf.as_slice()).unwrap();
    let expected = Treq { headers: Vec::new(), body: body() };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_treq(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_rreqok() {
    // Request type: RreqOk(1,BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
               0x6f, 0x72, 0x6c, 0x64, ];

    let msg = decode_rreq(buf.as_slice()).unwrap();
    let expected = Rmsg::Ok(body());

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rreq(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_rreqerror() {
    // Request type: RreqError(1,hello world)
    let buf = vec![0x01, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
               0x6f, 0x72, 0x6c, 0x64, ];

    let msg = decode_rreq(buf.as_slice()).unwrap();
    let expected = Rmsg::Error(BUFFER_STR.to_owned());

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rreq(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_rreqnack() {
    // Request type: RreqNack(1)
    let buf = vec![0x02, ];

    let msg = decode_rreq(buf.as_slice()).unwrap();
    let expected = Rmsg::Nack("".to_owned());

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rreq(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_tdispatch_1() {
    // Request type: Tdispatch(1,List(),Path(path),Dtab(),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x00, 0x00, 0x05, 0x2f, 0x70, 0x61, 0x74,
               0x68, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
               0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, ];

    let msg = decode_tdispatch(buf.as_slice()).unwrap();
    let expected = Tdispatch {
        contexts: Vec::new(),
        dest: "/path".to_owned(),
        dtab: Dtab::new(),
        body: body(),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_tdispatch(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
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
}

#[test]
fn test_tdispatch_3() {
    // Request type: Tdispatch(1,List(),Path(path),Dtab(/f/foo=>/go),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x00, 0x00, 0x05, 0x2f, 0x70, 0x61, 0x74,
               0x68, 0x00, 0x01, 0x00, 0x06, 0x2f, 0x66, 0x2f,
               0x66, 0x6f, 0x6f, 0x00, 0x03, 0x2f, 0x67, 0x6f,
               0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f,
               0x72, 0x6c, 0x64, ];

    let msg = decode_tdispatch(buf.as_slice()).unwrap();
    let mut dtab = Dtab::new();
    dtab.add_entry("/f/foo".to_owned(), "/go".to_owned());

    let expected = Tdispatch {
        contexts: vec![],
        dest: "/path".to_owned(),
        dtab: dtab,
        body: body(),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_tdispatch(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_rdispatchok_1() {
    // Request type: RdispatchOk(1,List(),BigEndianHeapChannelBuffer(ridx=0, widx=11, cap=11))
    let buf = vec![0x00, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
               0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, ];

    let msg = decode_rdispatch(buf.as_slice()).unwrap();
    let expected = Rdispatch {
        contexts: vec![],
        msg: Rmsg::Ok(body()),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rdispatch(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
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

    let msg = decode_rdispatch(buf.as_slice()).unwrap();
    let expected = Rdispatch {
        contexts: vec![(body(), body())],
        msg: Rmsg::Ok(body()),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rdispatch(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_rdispatcherror() {
    // Request type: RdispatchError(1,List(),hello world)
    let buf = vec![0x01, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f,
               0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, ];

    let msg = decode_rdispatch(buf.as_slice()).unwrap();
    let expected = Rdispatch {
        contexts: vec![],
        msg: Rmsg::Error(BUFFER_STR.to_owned()),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rdispatch(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_rdispatchnack() {
    // Request type: RdispatchNack(1,List())
    let buf = vec![0x02, 0x00, 0x00, ];

    let msg = decode_rdispatch(buf.as_slice()).unwrap();
    let expected = Rdispatch {
        contexts: vec![],
        msg: Rmsg::Nack("".to_owned()),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_rdispatch(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
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

    let msg = decode_tdiscarded(buf.as_slice()).unwrap();
    let expected = Tdiscarded {
        id: 1,
        msg: BUFFER_STR.to_owned(),
    };

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_tdiscarded(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}

#[test]
fn test_tlease() {
    // Request type: Tlease(0,1000)
    let buf = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, ];

    let msg = decode_tlease_duration(buf.as_slice()).unwrap();
    let expected = Duration::from_millis(1000);

    assert_eq!(msg, expected);

    let mut w = writer();
    encode_tlease_duration(&mut w, &expected).unwrap();

    assert_eq!(w.into_inner(), buf);
}
