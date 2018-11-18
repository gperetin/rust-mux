// Some way to start the server and know it's started
// Do we want to start it always on the same port?
// How do we stop the server when the tests are complete?
// Test Ping

extern crate mux;

use std::io;
use std::net::TcpStream;

use mux::session::MuxSession;

struct MuxClient {
    session: MuxSession
}

impl MuxClient {
    fn new(ip: &str) -> io::Result<MuxClient> {
        // Open socket
        // Give socket to session
        // save session in struct
        let socket = TcpStream::connect(ip).unwrap();
        let session = MuxSession::new(socket).unwrap();
        Ok(MuxClient { session })
    }

    fn ping(&self) -> bool {
        match self.session.ping() {
            Ok(_) => true,
            Err(_) => false
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
}
