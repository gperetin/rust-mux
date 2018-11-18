extern crate mux;

use std::io;
use std::net::TcpStream;

use mux::session::MuxSession;
use mux::Rmsg;

struct MuxClient {
    session: MuxSession,
}

impl MuxClient {
    fn new(ip: &str) -> io::Result<MuxClient> {
        let socket = TcpStream::connect(ip).unwrap();
        let session = MuxSession::new(socket).unwrap();
        Ok(MuxClient { session })
    }

    fn ping(&self) -> bool {
        match self.session.ping() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn dispatch(&self, payload: Vec<u8>) -> io::Result<Vec<u8>> {
        let frame = mux::Tdispatch::new("/foo".to_string(), payload);

        let msg = self.session.dispatch(&frame).unwrap();
        if let Rmsg::Ok(body) = msg.msg {
            Ok(body)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Error during mux request!"))
        }
    }
}

#[cfg(test)]
mod end_to_end {
    use super::MuxClient;

    #[test]
    fn can_send_ping() {
        let client = MuxClient::new("127.0.0.1:8080").unwrap();
        assert!(client.ping());
    }

    #[test]
    fn can_send_dispatch() {
        let client = MuxClient::new("127.0.0.1:8080").unwrap();
        let payload: Vec<u8> = String::from("some text").into_bytes();
        assert_eq!(client.dispatch(payload.clone()).unwrap(), payload);
    }
}
