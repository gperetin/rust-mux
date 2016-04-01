use super::*;

use std::io;


static TDISPATCH_BUF: &'static [u8] = &[
    0, 0, 0, 65, // frame size

    2, // TDISPATCH
    4, 7, 9, // tag

    // contexts:
    0, 2, // 2 contexts

    // context 0 key
    0, 4, // length
    1, 2, 3, 4,

    // context 0 val
    0, 2, // length
    6, 7,

    // context 1 key
    0, 2, // length
    3, 4,

    // context 1 val
    0, 3, // length
    6, 7, 8,

    // dst
    0, 4, // length
    '/' as u8, 66, 65, 68, // "/BAD"

    // dtab: /BAD => /DAD
    0, 1, // length
    0, 4, // source length
    '/' as u8, 66, 65, 68, // "/BAD"
    0, 4, // tree length
    '/' as u8, 68, 65, 68, // "/DAD"

    // data: [0 .. 20)
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];

fn new_write() -> io::Cursor<Vec<u8>> {
    io::Cursor::new(Vec::new())
}

fn roundtrip_frame(msg: MessageFrame) {
    let msg = Message {
        tag: Tag::new(true, 2),
        frame: msg,
    };

    let mut w = new_write();
    let _ = codec::encode_message(&mut w, &msg).unwrap();
    let w = w.into_inner();

    let mut two = Vec::new();

    two.extend_from_slice(&w);
    two.extend_from_slice(&w);

    let mut buffer = io::Cursor::new(two);

    // decode once
    let decoded = codec::read_message(&mut buffer).unwrap();
    assert_eq!(&msg, &decoded);

    // decode it again to make sure we didn't consume too much data the first time
    let decoded = codec::read_message(&mut buffer).unwrap();
    assert_eq!(&msg, &decoded);
}

#[test]
fn decode_tdispatch() {
    let msg = codec::read_message(&mut io::Cursor::new(TDISPATCH_BUF)).unwrap();
    let expected_tag = Tag::new(true, (4 << 16) | (7 << 8) | 9);

    assert_eq!(&expected_tag, &msg.tag);
}

#[test]
fn encode_rdispatch() {
    let msg_frame = MessageFrame::Rdispatch(Rdispatch {
        contexts: Vec::new(),
        msg: Rmsg::Ok(b"nope".to_vec()),
    });
    let tag = Tag::new(true, (1 << 16) | (2 << 8) | 3);
    let msg = Message { tag: tag, frame: msg_frame };

    let expected = vec![
        0x00, 0x00, 0x00, 0x0b, // frame
        0xfe, // msg type: rdispatch (-2)
        0x01, 0x02, 0x03, // tag
        0x00, // status
        0x00, 0x00, // contexts
        0x6e, 0x6f, 0x70, 0x65 // "nope""
        ];

    let mut bytes = io::Cursor::new(Vec::new());
    codec::encode_message(&mut bytes, &msg).unwrap();
    let bytes = bytes.into_inner();

    /* Test the encoding of the frame */ {
        assert_eq!(&bytes, &expected);
    }

    /* Test decoding the context of the frame */ {
        let mut read = io::Cursor::new(&bytes[8..]);
        let ctxs = codec::decode_contexts(&mut read).unwrap();
        assert_eq!(ctxs, vec![]);
    }

    /* Test decoding the tag of the frame */ {
        let mut read = io::Cursor::new(&bytes[5..]);
        let tag = codec::decode_tag(&mut read).unwrap();
        assert_eq!(&msg.tag, &tag);
    }
}

#[test]
fn roundtrip_treq() {
    roundtrip_frame(MessageFrame::Treq(Treq {
        headers: vec![(1, vec![4, 5, 6])],
        body: vec![1, 2, 3],
    }));

    roundtrip_frame(MessageFrame::Treq(Treq {
        headers: Vec::new(),
        body: Vec::new(),
    }));
}

#[test]
fn roundtrip_rreq() {
    roundtrip_frame(MessageFrame::Rreq(Rmsg::Ok(vec![1, 2, 3])));
    roundtrip_frame(MessageFrame::Rreq(Rmsg::Nack("Boo".to_owned())));
    roundtrip_frame(MessageFrame::Rreq(Rmsg::Error("Boo".to_owned())));
}

#[test]
fn roundtrip_tdispatch() {

    roundtrip_frame(MessageFrame::Tdispatch(Tdispatch {
        contexts: vec![(vec![1, 2, 3], vec![4, 5, 6])],
        dest: "foo".to_string(),
        dtab: Dtab::from(vec![("foo".to_string(), "bar".to_string())]),
        body: vec![1, 2, 3],
    }));

    roundtrip_frame(MessageFrame::Tdispatch(Tdispatch {
        contexts: Vec::new(),
        dest: "foo".to_string(),
        dtab: Dtab::new(),
        body: Vec::new(),
    }));

}

#[test]
fn roundtrip_rdispatch() {
    roundtrip_frame(MessageFrame::Rdispatch(Rdispatch {
        contexts: vec![(vec![1, 2, 3], vec![4, 5, 6])],
        msg: Rmsg::Ok(vec![1, 2, 3]),
    }));

    roundtrip_frame(MessageFrame::Rdispatch(Rdispatch {
        contexts: Vec::new(),
        msg: Rmsg::Nack("Boo".to_owned()),
    }));

    roundtrip_frame(MessageFrame::Rdispatch(Rdispatch {
        contexts: Vec::new(),
        msg: Rmsg::Error("Boo".to_owned()),
    }));
}

#[test]
fn roundtrip_tinit() {
    roundtrip_frame(MessageFrame::Tinit(Init {
        version: 12,
        headers: vec![(vec![1, 2, 3], vec![4, 5, 6, 7])],
    }));

    roundtrip_frame(MessageFrame::Tinit(Init {
        version: 1,
        headers: vec![(vec![43, 127], vec![])],
    }));
}

#[test]
fn roundtrip_rinit() {
    roundtrip_frame(MessageFrame::Rinit(Init {
        version: 12,
        headers: vec![(vec![1, 2, 3], vec![4, 5, 6, 7])],
    }));

    roundtrip_frame(MessageFrame::Rinit(Init {
        version: 1,
        headers: vec![(vec![43, 127], vec![])],
    }));
}

#[test]
fn roundtrip_tdrain() {
    roundtrip_frame(MessageFrame::Tdrain);
}

#[test]
fn roundtrip_rdrain() {
    roundtrip_frame(MessageFrame::Rdrain);
}

#[test]
fn roundtrip_tping() {
    roundtrip_frame(MessageFrame::Tping);
}

#[test]
fn roundtrip_rping() {
    roundtrip_frame(MessageFrame::Rping);
}

#[test]
fn roundtrip_rerr() {
    roundtrip_frame(MessageFrame::Rerr("Foo!".to_owned()));
}

#[test]
fn roundtrip_tlease() {
    use std::time::Duration;

    // note that this will fail for precision below 1 ms
    roundtrip_frame(MessageFrame::Tlease(Duration::new(123, 1_000_000)));
}

#[test]
fn roundtrip_dtab() {
    fn roundtrip_frame(table: &Dtab) {
        let mut w = new_write();
        let _ = codec::encode_dtab(&mut w, table).unwrap();
        let mut w = io::Cursor::new(w.into_inner());
        let decoded = codec::decode_dtab(&mut w).unwrap();

        assert_eq!(table, &decoded);
    }

    let mut tab = Dtab::new();

    roundtrip_frame(&tab);
    tab.add_entry("a".to_string(), "b".to_string());
    roundtrip_frame(&tab);
    tab.add_entry("c".to_string(), "d".to_string());
    roundtrip_frame(&tab);
}


#[test]
fn roundtrip_context() {
    fn roundtrip_frame(ctx: &Contexts) {
        let mut w = new_write();
        let _ = codec::encode_contexts(&mut w, ctx).unwrap();
        let mut w = io::Cursor::new(w.into_inner());
        let decoded = codec::decode_contexts(&mut w).unwrap();

        assert_eq!(ctx, &decoded);


    }

    roundtrip_frame(&Vec::new());
    roundtrip_frame(&vec![(vec![1, 2, 3], vec![4, 5, 6])]);
}


#[test]
fn roundtrip_tag() {
    fn roundtrip_frame(tag: &Tag) {
        let mut w = new_write();
        let _ = codec::encode_tag(&mut w, &tag).unwrap();
        let mut w = io::Cursor::new(w.into_inner());
        let decoded = codec::decode_tag(&mut w).unwrap();

        assert_eq!(tag, &decoded);
    }

    roundtrip_frame(&Tag {
        end: false,
        id: 1,
    });

    roundtrip_frame(&Tag { end: true, id: 1 });

    roundtrip_frame(&Tag {
        end: false,
        id: 0x0fffff,
    });
}
