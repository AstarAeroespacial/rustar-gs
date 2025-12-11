use crate::Demodulator;
use std::{
    io,
    path::{Path, PathBuf},
    process, thread,
    time::Duration,
};

// TODO: separate build and run steps
// TODO: implement cleanup to kill child process

// to where i send the samples for demodulation
const SAMPLE_SINK: &str = "tcp://127.0.0.1:5556";
// from where i read the demodulated bits
const BIT_SOURCE: &str = "tcp://127.0.0.1:5557";

#[derive(Debug)]
pub enum DemodulatorError {
    NoSuchFlowgraph,
    GnuRadioProcess(io::Error),
}

pub struct Afsk1200 {
    #[allow(dead_code)]
    python_proc: process::Child, // keep it alive
}

impl Afsk1200 {
    /// Create a new `Afsk1200` demodulator, with a valid GNU Radio flowgraph, and run the process.
    pub fn new(flowgraph_path: impl AsRef<Path>) -> Result<Self, DemodulatorError> {
        if !flowgraph_path.as_ref().exists() {
            return Err(DemodulatorError::NoSuchFlowgraph);
        }

        // let child = process::Command::new("python3")
        let python_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../modem/gnuradio/python");
        dbg!(&python_path);

        let child = process::Command::new(python_path)
            .arg(flowgraph_path.as_ref())
            .spawn()
            .map_err(DemodulatorError::GnuRadioProcess)?;

        Ok(Self { python_proc: child })
    }
}

pub struct Afsk1200Iterator<I>
where
    I: Iterator<Item = Vec<f64>>,
{
    inner: I,
    zmq_pub: zmq::Socket,
    zmq_sub: zmq::Socket,
}

impl<I> Afsk1200Iterator<I>
where
    I: Iterator<Item = Vec<f64>>,
{
    fn new(inner: I) -> Self {
        let ctx = zmq::Context::new();

        let pub_sock = ctx.socket(zmq::PUB).unwrap();
        pub_sock.bind(SAMPLE_SINK).unwrap();

        let sub_sock = ctx.socket(zmq::SUB).unwrap();
        sub_sock.connect(BIT_SOURCE).unwrap();
        sub_sock.set_subscribe(b"").unwrap();
        sub_sock.set_rcvtimeo(1000).unwrap();

        // This sleep is needed to give time to the GNU Radio process to subscribe.
        // TODO: there must be a better way. Maybe if I do the wrapper crate I can handle this better.
        thread::sleep(Duration::from_millis(200));

        Self {
            inner,
            zmq_pub: pub_sock,
            zmq_sub: sub_sock,
        }
    }
}

impl<I> Iterator for Afsk1200Iterator<I>
where
    I: Iterator<Item = Vec<f64>>,
{
    type Item = Vec<bool>;

    // TODO: handle errors, timeout... Return Option<Result<...>> ?
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(samples) = self.inner.next() {
            let byte_slice = unsafe {
                std::slice::from_raw_parts(
                    samples.as_ptr() as *const u8,
                    samples.len() * std::mem::size_of::<f64>(),
                )
            };

            self.zmq_pub.send(byte_slice, 0).unwrap();
        }

        if let Ok(msg) = self.zmq_sub.recv_bytes(0) {
            Some(msg.iter().map(|b| *b != 0).collect())
        } else {
            None
        }
    }
}

impl<I> Demodulator<I> for Afsk1200
where
    I: Iterator<Item = Vec<f64>>,
{
    type Output = Afsk1200Iterator<I>;

    fn bits(&self, input: I) -> Self::Output {
        Afsk1200Iterator::new(input)
    }
}
