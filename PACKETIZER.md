# Packetizer Implementation

This document describes the packetizer implementation that addresses the requirement: "We have to implement the Iterator that takes frames (HDLC, AX.25) and yields telemetry packets."

## Overview

The packetizer is implemented as an Iterator that converts frames into telemetry packets with a simple one-to-one mapping. It provides lazy evaluation, allowing efficient processing of large streams of frames.

## Implementation Details

### Core Components

- **`Packetizer<I>`**: An iterator struct that takes an iterator of frames and yields telemetry packets
- **`packetize()`**: A convenience function for creating packetizer instances
- **Integration**: The existing `deframe()` function now uses the packetizer internally

### File Structure

```
hdlc/src/
├── packetizer.rs       # New packetizer implementation
├── deframer.rs         # Updated to use packetizer
└── lib.rs              # Updated to expose packetizer module
```

## Usage

The packetizer implements the Iterator trait, so it can be used with all standard iterator methods:

```rust
use hdlc::packetizer::packetize;

// Convert frames to packets using iterator
let frames = /* iterator of Frame objects */;
let packets: Vec<TelemetryPacket> = packetize(frames).collect();

// Or process lazily
let mut packetizer = packetize(frames);
let first_packet = packetizer.next(); // Only processes one frame
let next_five: Vec<_> = packetizer.take(5).collect(); // Processes 5 more frames
```

## Features

### Simple Telemetry Encoding
- One telemetry packet per frame (as specified in requirements)
- Filters out frames without info data
- Handles invalid frame data gracefully

### Iterator Interface
- Implements the standard `Iterator` trait
- Provides lazy evaluation - frames are processed only when packets are requested
- Compatible with all iterator combinators (`map`, `filter`, `take`, etc.)

### Error Handling
- Silently skips frames that cannot be converted to telemetry packets
- Continues processing remaining frames if one frame is invalid

## Testing

The implementation includes comprehensive tests:

- **Unit tests**: 5 tests covering various scenarios (empty input, valid frames, invalid data, etc.)
- **Integration tests**: Verifies integration with existing deframing functionality
- **All existing tests pass**: Maintains backward compatibility

### Test Coverage

- Empty frame sequences
- Single valid frames
- Multiple frames with mixed validity
- Frames without info data
- Invalid telemetry data
- Lazy evaluation behavior

## Backward Compatibility

The existing `deframe()` function has been updated to use the packetizer internally:

```rust
fn deframe(frames: Vec<Frame>) -> Vec<TelemetryPacket> {
    packetize(frames.into_iter()).collect()
}
```

This ensures that all existing code continues to work unchanged while providing the new iterator-based interface.

## Performance

- **Lazy evaluation**: Frames are processed only when packets are requested
- **Memory efficient**: No intermediate collections for large frame streams  
- **Zero-copy**: Direct iterator chaining without additional allocations