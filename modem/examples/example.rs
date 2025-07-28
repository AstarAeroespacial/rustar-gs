use std::{fs::File, path::PathBuf, thread::sleep, time::Duration};

use modem::Demodulator;

extern crate modem;

fn main() {
    let reader = File::open("./out_upsampled_40.txt").unwrap();
    let writer = File::create("./demodulated_bits.txt").unwrap();

    let flowgraph_name = "afsk_demod_headless";
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let demod = Demodulator::build(
        reader,
        writer,
        here.join("flowgraphs")
            .join(format!("{}.py", flowgraph_name)),
        Some(here.join("gnuradio/python")),
    )
    .unwrap();

    sleep(Duration::from_millis(500)); // magic sleep. TODO: small handshake to ensure the flowgraph is ready.

    demod.run();
}
