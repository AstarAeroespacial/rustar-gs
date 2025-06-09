use std::{
    io::{Read, Seek, Write},
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

// use zmq::Socket;

// bytemuck for zero copy conversions, may me more performant

// async?? custom traits "SampleSource" and "BitSink"
// abstract for any reader or writer?
pub struct Demodulator<R: Read + Seek + Send + 'static, W: Write + Send + 'static> {
    // struct Demodulator {
    // pub reader: Arc<mpsc::Receiver<Bytes>>,
    pub reader: R,
    pub writer: W,
    // pub writer: Arc<mpsc::Sender<Bit>>,
    // sink: Socket,
    // source: Socket,
}

type Sample = [f32; 2];
type Bit = bool;

const SAMPLE_SINK: &str = "tcp://127.0.0.1:5556";
const BIT_SOURCE: &str = "tcp://127.0.0.1:5557";

const BATCH_SIZE: usize = 128; // Number of samples per batch

impl<R: Read + Seek + Send + 'static, W: Write + Send + 'static> Demodulator<R, W> {
    pub fn build(reader: R, writer: W) -> Self {
        //     let subscriber = context.socket(zmq::SUB).unwrap();
        //     subscriber.connect(TELEMETRY_SUB_ADDR).unwrap();
        //     subscriber.set_subscribe(b"").unwrap();

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

                if n == 0 {
                    self.reader.rewind().unwrap();
                }

                //dbg!(n);
                assert!(n % 8 == 0); // must receive discrete number of samples

                publisher.send(&buffer[0..n], 0).unwrap();
            }
        });

        let receiver = thread::spawn(move || {
            let context = zmq::Context::new();
            let subscriber = context.socket(zmq::SUB).unwrap();
            subscriber.connect(BIT_SOURCE).unwrap();
            subscriber.set_subscribe(b"").unwrap();

            loop {
                let msg = subscriber.recv_bytes(0).unwrap();
                dbg!(&msg);

                self.writer.write(&msg).unwrap();
            }
        });

        sender.join().unwrap();
        receiver.join().unwrap();
    }
}

// loop {
//     self.reader.read_exact(&mut buffer).unwrap();
//     self.sink.send(&buffer, 0).unwrap();
// }

// unimplemented!()

#[cfg(test)]
mod tests {
    use super::*;
}
