# SDR Module

Software Defined Radio (SDR) interface for the RUSTAR ground station.

## Overview

This module provides a trait-based interface for different SDR implementations, allowing the ground station to receive IQ samples from various sources.

## Implementations

### MockSdr
Generates synthetic sine wave signals in baseband IQ format. Useful for testing the signal processing pipeline without hardware.

```rust
let sdr = MockSdr::new(48_000.0, 1200.0, 512);
```

### ZmqMockSdr
Receives IQ samples from a running GNU Radio flowgraph via ZeroMQ (ZMQ). This allows you to use GNU Radio's signal processing blocks and hardware interfaces while integrating with the Rust-based ground station.

**Configuration:**
```toml
[sdr]
type = "zmq_mock"
zmq_endpoint = "tcp://127.0.0.1:5556"
```

**GNU Radio Setup:**
1. Create your GNU Radio flowgraph with signal source or SDR hardware
2. Add a **ZMQ PUB Sink** block to publish IQ samples
3. Configure the ZMQ PUB Sink:
   - Address: `tcp://*:5556` (or your chosen port)
   - Socket Type: PUB
4. Run the GNU Radio flowgraph
5. Start the ground station - it will connect and receive samples

The ZmqMockSdr receives raw bytes from the ZMQ socket and converts them to `f64` samples for processing.

## Usage

The SDR runs in an async task that:
- Receives frequency control commands via a Tokio channel
- Continuously reads IQ samples from the SDR
- Sends samples to the demodulator via a standard channel

```rust
let sdr = create_sdr(&config.sdr);
let (cmd_tx, cmd_rx) = mpsc::channel(1);
let (samp_tx, samp_rx) = std::sync::mpsc::channel();

tokio::spawn(sdr_task(sdr, cmd_rx, samp_tx));
```
