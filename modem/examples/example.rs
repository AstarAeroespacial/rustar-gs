use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::mpsc,
    thread,
};

use modem::Demodulator;

extern crate modem;

const SAMPLES_BATCH_SIZE: usize = 10_000;

fn main() {
    /* let mut reader = File::open("./modem/examples/samples.iq").unwrap();
    let mut writer = File::create("./modem/examples/output.bit").unwrap();

    let flowgraph_name = "afsk_demod";
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (tx, rx) = mpsc::channel();

    let (tx_samples, rx_bits) = mpsc::channel();

    let demod = Demodulator::build(
        rx,
        tx_samples,
        here.join("flowgraphs")
            .join(format!("{}.py", flowgraph_name)),
        //Some(here.join("gnuradio/python")),
        None::<PathBuf>,
    )
    .unwrap();

    //sleep(Duration::from_millis(500)); // magic sleep. TODO: small handshake to ensure the flowgraph is ready.

    let demod_handler = thread::spawn(move || {
        demod.run();
    });

    let receiver_handler = thread::spawn(move || {
        while let Ok(bit_batch) = rx_bits.recv() {
            let bits = bit_batch
                .iter()
                .map(|b| if *b { b'1' } else { b'0' })
                .collect::<Vec<_>>();
            writer.write_all(&bits).unwrap();
            dbg!(&bit_batch);
        }
    });

    let mut samples = Vec::new();

    loop {
        let mut buffer = [0u8; 8];
        let n = reader.read(&mut buffer).unwrap();

        if n == 0 {
            break;
        }

        if n != 8 {
            panic!("unexpected partial sample");
        }

        samples.push(buffer);

        // send SAMPLES_BATCH_SIZE samples
        if samples.len() == SAMPLES_BATCH_SIZE {
            dbg!(&samples.len());
            tx.send(samples).unwrap();
            samples = Vec::new();
        }
    }

    // drop the tx end to signal the receiver there is no more samples
    drop(tx);

    demod_handler.join().unwrap();
    receiver_handler.join().unwrap(); */
}
