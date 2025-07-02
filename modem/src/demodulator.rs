use std::{
    env,
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command},
    thread,
};

// bytemuck for zero copy conversions, may me more performant

// async?? custom traits "SampleSource" and "BitSink"
// abstract for any reader or writer?
pub struct Demodulator<R: Read + Send + 'static, W: Write + Send + 'static> {
    // from where the samples are read
    pub reader: R,
    // to where we write the bits
    pub writer: W,
    #[allow(dead_code)]
    python_proc: Child, // keep it alive
}

type Sample = [f32; 2];
type Bit = bool;

// to where i send the samples for demodulation
const SAMPLE_SINK: &str = "tcp://127.0.0.1:5556";
// from where i read the demodulated bits
const BIT_SOURCE: &str = "tcp://127.0.0.1:5557";

// I LOWERED IT BECAUSE WITH SMALL FILES SOMETIMES NOTHING IS
// SENT / RECEIVED, I HAVE TO INVESTIGATE IT FURTHER
// TODO: add telemetry/counters
// TODO: higher batch size
const BATCH_SIZE: usize = 2; // Number of samples per batch

// TODO: implement Drop and kill child.

impl<R: Read + Send + 'static, W: Write + Send + 'static> Demodulator<R, W> {
    // TODO: can fail!!
    pub fn build(reader: R, writer: W) -> Self {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let child = Command::new(dir.join("gnuradio/python"))
            .arg(dir.join("flowgraphs/afsk_demod_headless.py"))
            .spawn()
            .expect("Failed to run GNU radio flowgraph");

        Self {
            // sink: publisher,
            // source: subscriber,
            reader: reader,
            writer: writer,
            python_proc: child,
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

                self.writer.flush().unwrap(); // TODO: really necessary?
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
