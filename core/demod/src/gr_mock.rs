const BIT_SOURCE: &str = "tcp://127.0.0.1:5557";

pub struct GrBitSource {
    sub_sock: zmq::Socket,
}

impl Default for GrBitSource {
    fn default() -> Self {
        Self::new()
    }
}

impl GrBitSource {
    pub fn new() -> Self {
        let ctx = zmq::Context::new();

        let sub_sock = ctx.socket(zmq::SUB).unwrap();
        sub_sock.connect(BIT_SOURCE).unwrap();
        sub_sock.set_subscribe(b"").unwrap();
        sub_sock.set_rcvtimeo(1000).unwrap();

        Self { sub_sock }
    }
}

impl Iterator for GrBitSource {
    type Item = Vec<bool>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(msg) = self.sub_sock.recv_bytes(0) {
            Some(msg.iter().map(|b| *b != 0).collect())
        } else {
            None
        }
    }
}
