use std::{
    env,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
};

// TODO: add telemetry/counters with batch sizes and stuff like that.
// TODO: abstraction or more generic interface for reader/writer, so we can use channels for example
pub struct Demodulator<R: Read + Send + 'static, W: Write + Send + 'static> {
    // from where the samples are read, as a stream of u8
    pub reader: R,
    // to where we write the bits, as a stream of bytes (0u8 or 1u8)
    pub writer: W,

    #[allow(dead_code)]
    python_proc: Child, // keep it alive
}

// to where i send the samples for demodulation
const SAMPLE_SINK: &str = "tcp://127.0.0.1:5556";
// from where i read the demodulated bits
const BIT_SOURCE: &str = "tcp://127.0.0.1:5557";

// LOWER WHEN TESTING WITH SMALL FILES, OTHERWISE SOMETIMES NOTHING
// IS SENT/RECEIVED. INVESTIGATE.
const BATCH_SIZE: usize = 2; // number of samples per batch received from gr

// TODO: implement Drop and kill child Python process.

// TODO: improve error reporting
#[derive(Debug)]
pub enum DemodulatorError {
    NoSuchFlowgraph,
    GnuRadioProcess,
}

// TODO: builder OR optional method to add python path
// TODO: builder, build on one step, begin execution of the flowgraph on another
impl<R: Read + Send + 'static, W: Write + Send + 'static> Demodulator<R, W> {
    // TODO: can fail!!
    pub fn build(
        reader: R,
        writer: W,
        flowgraph_path: impl AsRef<Path>,
        python_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, DemodulatorError> {
        // Use `python_path`, or whatever `python` is in `$PATH`.
        let python = python_path
            .map(|p| p.as_ref().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("python"));

        if !flowgraph_path.as_ref().exists() {
            return Err(DemodulatorError::NoSuchFlowgraph);
        }

        let child = Command::new(&python)
            .arg(&flowgraph_path.as_ref())
            .spawn()
            .map_err(|_| DemodulatorError::GnuRadioProcess)?;

        Ok(Self {
            reader,
            writer,
            python_proc: child,
        })
    }

    pub fn run(mut self) {
        let sender = thread::spawn(move || {
            let context = zmq::Context::new();
            let gr_samples_publisher = context.socket(zmq::PUB).unwrap();
            gr_samples_publisher.bind(SAMPLE_SINK).unwrap();

            let mut buffer = [0u8; 8 * BATCH_SIZE]; // samples batch

            loop {
                let n = self.reader.read(&mut buffer).unwrap();

                assert!(n % 8 == 0); // must receive discrete number of samples

                if n > 0 {
                    gr_samples_publisher.send(&buffer[0..n], 0).unwrap();
                }
            }
        });

        let receiver = thread::spawn(move || {
            let context = zmq::Context::new();
            let gr_radio_bits_subscriber = context.socket(zmq::SUB).unwrap();
            gr_radio_bits_subscriber.connect(BIT_SOURCE).unwrap();
            gr_radio_bits_subscriber.set_subscribe(b"").unwrap();

            loop {
                let msg = gr_radio_bits_subscriber.recv_bytes(0).unwrap();

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
