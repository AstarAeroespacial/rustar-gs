use std::{
    io::{Read, Write},
    thread,
};

pub struct Modulator<R: Read + Send + 'static, W: Write + Send + 'static> {
    pub reader: R, // read bits to modulate
    pub writer: W, // write samples
}

impl<R: Read + Send + 'static, W: Write + Send + 'static> Modulator<R, W> {
    pub fn build(reader: R, writer: W) -> Self {
        Self {
            reader: reader,
            writer: writer,
        }
    }

    pub fn run(self) {
        let sender = thread::spawn(move || {
            // send bits to GNU radio for modulation
        });

        let receiver = thread::spawn(move || {
            // get samples from
        });
    }
}
