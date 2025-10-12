use std::f64::consts::PI;

pub trait Sdr {
    fn set_rx_frequency(&mut self, frequency: f64);
    fn read_samples(&mut self) -> Option<Vec<f64>>;
}

/// Mock SDR that generates a synthetic sine wave in baseband IQ
pub struct MockSdr {
    sample_rate: f64,
    freq: f64,
    phase: f64,
    block_size: usize,
}

impl MockSdr {
    pub fn new(sample_rate: f64, freq: f64, block_size: usize) -> Self {
        Self {
            sample_rate,
            freq,
            phase: 0.0,
            block_size,
        }
    }
}

impl Sdr for MockSdr {
    fn set_rx_frequency(&mut self, freq_hz: f64) {
        println!("[SDR] Setting frequency to {}", freq_hz);

        self.freq = freq_hz;
    }

    fn read_samples(&mut self) -> Option<Vec<f64>> {
        let mut out = Vec::with_capacity(2 * self.block_size);
        let phase_inc = 2.0 * PI * self.freq / self.sample_rate;

        for _ in 0..self.block_size {
            let i = self.phase.cos();
            let q = self.phase.sin();
            out.push(i);
            out.push(q);
            self.phase = (self.phase + phase_inc) % (2.0 * PI);
        }

        // println!("[SDR] Pushing samples");

        Some(out)
    }
}

pub struct ZmqMockSdr {
    sub_sock: zmq::Socket,
}

impl ZmqMockSdr {
    pub fn new(endpoint: String) -> Self {
        let ctx = zmq::Context::new();
        let sub_sock = ctx.socket(zmq::SUB).unwrap();
        sub_sock.connect(&endpoint).unwrap();
        sub_sock.set_subscribe(b"").unwrap();
        sub_sock.set_rcvtimeo(1000).unwrap();

        Self { sub_sock }
    }
}

impl Sdr for ZmqMockSdr {
    fn set_rx_frequency(&mut self, freq_hz: f64) {
        println!("[ZMQ SDR] Setting frequency to {} Hz", freq_hz);
    }

    fn read_samples(&mut self) -> Option<Vec<f64>> {
        if let Ok(msg) = self.sub_sock.recv_bytes(0) {
            Some(msg.iter().map(|&b| b as f64).collect())
        } else {
            None
        }
    }
}

impl Sdr for Box<dyn Sdr + Send> {
    fn set_rx_frequency(&mut self, frequency: f64) {
        (**self).set_rx_frequency(frequency)
    }

    fn read_samples(&mut self) -> Option<Vec<f64>> {
        (**self).read_samples()
    }
}

pub enum SdrCommand {
    SetRxFrequency(f64),
}

pub async fn sdr_task(
    mut sdr: impl Sdr,
    mut control_rx: tokio::sync::mpsc::Receiver<SdrCommand>,
    samples_tx: std::sync::mpsc::Sender<Vec<f64>>, // normal channel, not async, unbounded
) {
    println!("[SDR TASK] Start");

    // NOTE: maybe read_samples() can be async and we can use in a select! arm
    loop {
        match control_rx.try_recv() {
            Ok(SdrCommand::SetRxFrequency(freq)) => {
                sdr.set_rx_frequency(freq);
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                // no command right now, go generate samples
                if let Some(samples) = sdr.read_samples() {
                    // NOTE: We'll have to be careful with this. The std channel sender is not
                    // blocking so it's okay to use here, but if we move to a _bounded_ channel,
                    // then we will have to consider making it async bc it blocks when full.
                    samples_tx.send(samples).unwrap(); // panicked here once
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
        }
    }
}
