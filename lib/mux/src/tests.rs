extern crate sharedbuffer;

use super::*;
use sharedbuffer::*;

use std::io;

fn new_write() -> io::Cursor<Vec<u8>> {
    io::Cursor::new(Vec::new())
}

#[test]
fn roundtrip_tdispatch() {
    fn tester(msg: &Tdispatch) {
        let mut w = new_write();
        let _ = frames::encode_tdispatch(&mut w, msg).unwrap();
        let w = SharedReadBuffer::new(w.into_inner());
        let decoded = frames::decode_tdispatch(w).unwrap();

        assert_eq!(msg, &decoded);

    }

    tester(&Tdispatch {
        contexts: Vec::new(),
        dest    : "foo".to_string(),
        dtable  : DTable::new(),
        body    : SharedReadBuffer::empty(),
    });
}

#[test]
fn roundtrip_dtable() {
    fn tester(table: &DTable) {
        let mut w = new_write();
        let _ = frames::encode_dtable(&mut w, table).unwrap();
        let mut w = io::Cursor::new(w.into_inner());
        let decoded = frames::decode_dtable(&mut w).unwrap();

        assert_eq!(table, &decoded);
    }

    let mut tab = DTable::new();

    tester(&tab);
    tab.add_entry("a".to_string(), "b".to_string());
    tester(&tab);
    tab.add_entry("c".to_string(), "d".to_string());
    tester(&tab);
}


#[test]
fn roundtrip_context() {
    fn tester(ctx: &Contexts) {
        let mut w = new_write();
        let _ = frames::encode_contexts(&mut w, ctx).unwrap();
        let mut w = io::Cursor::new(w.into_inner());
        let decoded = frames::decode_contexts(&mut w).unwrap();

        assert_eq!(ctx, &decoded);


    }

    tester(&Vec::new());
    tester(&vec![(vec![1,2,3],vec![4,5,6])]);
}


#[test]
fn roundtrip_tag() {
    fn tester(tag: &Tag) {
        let mut w = new_write();
        let _ = encode_tag(&mut w, &tag).unwrap();
        let mut w = io::Cursor::new(w.into_inner());
        let decoded = decode_tag(&mut w).unwrap();

        assert_eq!(tag, &decoded);
    } 

    tester(&Tag {
        end: false,
        id: 1,
    });

    tester(&Tag {
        end: true,
        id: 1,
    });

    tester(&Tag {
        end: false,
        id: 0x0fffff,
    });
}

