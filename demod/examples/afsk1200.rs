use demod::{Demodulator, afsk1200::Afsk1200};
use itertools::Itertools;

use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

const SAMPLES_BATCH_SIZE: usize = 10_000;

fn main() {
    println!("Running afsk_demod.py");

    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flowgraph_path = here.join("flowgraphs").join("afsk_demod.py");

    let demod = Afsk1200::new(flowgraph_path).unwrap();

    let reader = BufReader::new(File::open(here.join("examples").join("samples.iq")).unwrap());

    // Convert the input bytes to f64 batches.
    let samples: Vec<Vec<f64>> = reader
        .bytes()
        .map(|b| b.unwrap())
        .chunks(8) // 8 bytes per f64
        .into_iter()
        .map(|chunk| {
            let bytes: [u8; 8] = chunk.collect::<Vec<u8>>().try_into().unwrap();
            f64::from_ne_bytes(bytes)
        })
        .chunks(SAMPLES_BATCH_SIZE)
        .into_iter()
        .map(|batch| batch.collect::<Vec<f64>>())
        .collect();

    let bits_iter = demod.bits(samples.into_iter());

    let mut writer = BufWriter::new(File::create("./demod/examples/output.bit").unwrap());

    for bits in bits_iter {
        // write bits as 0/1 bytes
        let bytes: Vec<u8> = bits
            .into_iter()
            .map(|b| if b { b'1' } else { b'0' })
            .collect();
        writer.write_all(&bytes).unwrap();
    }

    println!("Finished demodulation. Bits written to output.bit");

    writer.flush().unwrap();
}
