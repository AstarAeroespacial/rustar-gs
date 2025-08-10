`examples.rs` shows a way to use the demodulator.

It uses a `samples.iq` input file, which must be a raw samples file of an AFSK transmission, captured as float32 (like the RTL-SDR does), for that is what the demodulator supports, for now.

As the demodulation flowgraph, it uses "afsk_demod".

A simple way to create this file is, given a raw samples file, created with GNU radio for example, run:

```sh
head -c 10000000 capture.iq > samples.iq
```

which will get the first ~10MB of samples.

A `samples.iq` file is provided, which should be demodulated to a sequence of "01111110" (the HDLC sync flag).


Run the example from the root of the project with:

```sh
cargo run -p modem --example example
```

The example will log some stuff, and store the demodulated bits in `mode/examples/output.bit`.
