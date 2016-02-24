use super::*;
use std::io;

fn new_write() -> io::Cursor<Vec<u8>> {
    io::Cursor::new(Vec::new())
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

