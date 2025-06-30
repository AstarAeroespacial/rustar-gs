use std::{fs::File, thread::sleep, time::Duration};

use modem::Demodulator;

extern crate modem;

fn main() {
    let reader = File::open("./out_upsampled_40.txt").unwrap();
    let writer = File::create("./demodulated_bits.txt").unwrap();

    let demod = Demodulator::build(reader, writer);

    sleep(Duration::from_millis(500)); // magic sleep. TODO: small handshake to ensure the flowgraph is ready.

    demod.run();
}
