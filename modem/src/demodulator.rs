use std::{
    io::{Read, Write},
    thread,
};

// bytemuck for zero copy conversions, may me more performant

// async?? custom traits "SampleSource" and "BitSink"
// abstract for any reader or writer?
pub struct Demodulator<R: Read + Send + 'static, W: Write + Send + 'static> {
    pub reader: R,
    pub writer: W,
}

type Sample = [f32; 2];
type Bit = bool;

const SAMPLE_SINK: &str = "tcp://127.0.0.1:5556";
const BIT_SOURCE: &str = "tcp://127.0.0.1:5557";

const BATCH_SIZE: usize = 128; // Number of samples per batch

impl<R: Read + Send + 'static, W: Write + Send + 'static> Demodulator<R, W> {
    pub fn build(reader: R, writer: W) -> Self {
        Self {
            // sink: publisher,
            // source: subscriber,
            reader: reader,
            writer: writer,
        }
    }

    pub fn run(mut self) {
        let sender = thread::spawn(move || {
            let context = zmq::Context::new();
            let publisher = context.socket(zmq::PUB).unwrap();
            publisher.bind(SAMPLE_SINK).unwrap();

            let mut buffer = [0u8; 8 * BATCH_SIZE]; // samples batch

            loop {
                let n = self.reader.read(&mut buffer).unwrap();

                assert!(n % 8 == 0); // must receive discrete number of samples

                if n > 0 {
                    publisher.send(&buffer[0..n], 0).unwrap();
                }
            }
        });

        let receiver = thread::spawn(move || {
            let context = zmq::Context::new();
            let subscriber = context.socket(zmq::SUB).unwrap();
            subscriber.connect(BIT_SOURCE).unwrap();
            subscriber.set_subscribe(b"").unwrap();

            loop {
                let msg = subscriber.recv_bytes(0).unwrap();

                for byte in msg {
                    let bit_char = if byte == 0 { b'0' } else { b'1' };
                    self.writer.write_all(&[bit_char]).unwrap();
                }
            }
        });

        sender.join().unwrap();
        receiver.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
