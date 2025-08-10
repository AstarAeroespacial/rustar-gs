use std::{
    io,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::mpsc::{Receiver, Sender},
    thread,
};

// Each IQ sample is an imaginary number.
// Both real and imaginary parts are an f32.
// TODO: this may not be true for all SDR frontends... make generic.
//type Sample = [f32; 2];
type Sample = [u8; 8];

// For now, bits are bools.
type Bit = bool;

// TODO: benchmark/check impact or memory usage sending owned Sample/Bit chunks, instead of
// refs/slices.

// TODO: add telemetry/counters with batch sizes and stuff like that.
// Count samples received and bits sent.
pub struct Demodulator {
    // from where the samples are read, as batches of [u8; 8]
    pub reader: Receiver<Vec<Sample>>,
    // to where we write the bits, as batches of bools
    pub writer: Sender<Vec<Bit>>,

    #[allow(dead_code)]
    python_proc: Child, // keep it alive
}

// to where i send the samples for demodulation
const SAMPLE_SINK: &str = "tcp://127.0.0.1:5556";
// from where i read the demodulated bits
const BIT_SOURCE: &str = "tcp://127.0.0.1:5557";

// TODO: implement Drop and kill child Python process.

// TODO: improve error reporting
#[derive(Debug)]
pub enum DemodulatorError {
    NoSuchFlowgraph,
    GnuRadioProcess(io::Error),
}

// TODO: builder OR optional method to add python path
// TODO: builder, build on one step, begin execution of the flowgraph on another
impl Demodulator {
    // TODO: can fail!!
    pub fn build(
        reader: Receiver<Vec<Sample>>,
        writer: Sender<Vec<Bit>>,
        flowgraph_path: impl AsRef<Path>,
        python_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, DemodulatorError> {
        // Use `python_path`, or whatever `python` is in `$PATH`.
        let python = python_path
            .map(|p| p.as_ref().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("python3"));

        if !flowgraph_path.as_ref().exists() {
            return Err(DemodulatorError::NoSuchFlowgraph);
        }

        let child = Command::new(&python)
            .arg(&flowgraph_path.as_ref())
            .spawn()
            .map_err(DemodulatorError::GnuRadioProcess)?;

        Ok(Self {
            reader,
            writer,
            python_proc: child,
        })
    }

    pub fn run(self) {
        // sends the samples to the flowgraph via zmq
        let sender_handle = thread::spawn(move || {
            let context = zmq::Context::new();
            let gr_samples_publisher = context.socket(zmq::PUB).unwrap();
            gr_samples_publisher.bind(SAMPLE_SINK).unwrap();

            while let Ok(samples) = self.reader.recv() {
                // reinterpret the samples received as Vec<[u8; 8]> as a &[u8]
                // TODO: bytemuck or something like that for safer conversions
                let byte_slice = unsafe {
                    std::slice::from_raw_parts(
                        samples.as_ptr() as *const u8,
                        samples.len() * std::mem::size_of::<Sample>(),
                    )
                };

                gr_samples_publisher.send(byte_slice, 0).unwrap();
            }

            dbg!("sender hung up");
        });

        // receives the bits from the flowgraph via zmq
        let receiver_handle = thread::spawn(move || {
            let context = zmq::Context::new();
            let gr_radio_bits_subscriber = context.socket(zmq::SUB).unwrap();
            gr_radio_bits_subscriber.connect(BIT_SOURCE).unwrap();
            gr_radio_bits_subscriber.set_subscribe(b"").unwrap();
            // timeout for exiting the receiver when transmission ends
            // TODO: find a better mechanism for this. shutdown() mwthod?
            gr_radio_bits_subscriber.set_rcvtimeo(1000).unwrap();

            while let Ok(msg) = gr_radio_bits_subscriber.recv_bytes(0) {
                let bit_batch = msg
                    .iter()
                    .map(|byte| if *byte == 0 { false } else { true })
                    .collect();

                self.writer.send(bit_batch).unwrap();
            }

            dbg!("timeout: nothing else from flowgraph");
        });

        sender_handle.join().unwrap();
        receiver_handle.join().unwrap();
    }
}
