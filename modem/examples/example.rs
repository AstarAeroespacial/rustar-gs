use std::{fs::File, io};

use modem::Demodulator;

extern crate modem;

fn main() {
    let reader = File::open("./out_upsampled_40.txt").unwrap();
    let writer = File::create("./demodulated_bits.txt").unwrap();

    let demod = Demodulator::build(reader, writer);
    demod.run();
}
