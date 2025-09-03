use crate::bitvecdeque::BitVecDeque;
use crate::frame::Bit;
use crate::frame::Frame;
use crate::packets::telemetry::TelemetryPacket;
use std::sync::mpsc::{Receiver, Sender};

pub trait Deframer {
    fn run(&mut self);
}

// Typical HDLC frames are up to 260 bytes (2080 bits)
// 4096 bits (512 bytes) is a safe upper bound for most use cases
const MAX_BUFFER_LEN: usize = 4096;
const FLAG_ARRAY: [Bit; 8] = [false, true, true, true, true, true, true, false];

pub enum ParserState {
    SearchingStartSync,
    SearchingEndSync,
}

pub struct HdlcDeframer {
    reader: Receiver<Vec<Bit>>,
    writer: Sender<Vec<TelemetryPacket>>,
    buffer: BitVecDeque,
    idx: usize, // it's an index, or offset, from the 0th element of the buffer
    state: ParserState,
}

impl HdlcDeframer {
    pub fn new(rx: Receiver<Vec<Bit>>, tx: Sender<Vec<TelemetryPacket>>) -> Self {
        Self {
            reader: rx,
            writer: tx,
            buffer: BitVecDeque::new(),
            idx: 0,
            state: ParserState::SearchingStartSync,
        }
    }

    fn find_frames(&mut self) -> Vec<Frame> {
        let mut frames = Vec::new();

        while self.buffer.len() - self.idx >= 8 {
            // Drop buffer if it grows too large (prevents DoS via never-ending garbage)
            if self.buffer.len() > MAX_BUFFER_LEN {
                self.buffer.clear();
                self.idx = 0;
                self.state = ParserState::SearchingStartSync;
                break;
            }
            // Usar slice_to_bitvec para obtener 8 bits y convertir a Vec<Bit>
            let bitvec_slice = self.buffer.slice_to_bitvec(self.idx, self.idx + 8);
            let slice: Vec<Bit> = bitvec_slice.iter().map(|bit| *bit).collect();

            // found sync
            if slice == FLAG_ARRAY {
                match self.state {
                    ParserState::SearchingStartSync => {
                        self.state = ParserState::SearchingEndSync;
                        // update the index, so it starts looking for the end sync
                        self.idx += 8;
                    }
                    ParserState::SearchingEndSync => {
                        // drain the whole frame between syncs
                        let frame_bits = self.buffer.drain_range(0, self.idx + 8);

                        if let Ok(frame) = Frame::try_from(frame_bits) {
                            frames.push(frame);
                        }

                        self.idx = 0;
                        self.state = ParserState::SearchingStartSync
                    }
                }
            }
            // if it's not a sync
            else {
                match self.state {
                    ParserState::SearchingStartSync => {
                        // drop the first element, it will be lost, but alas, such is life...
                        // this is something to try to avoid
                        self.buffer.pop_front();
                    }
                    ParserState::SearchingEndSync => {
                        // increment the index to continue looking
                        self.idx += 1;
                    }
                }
            }
        }
        frames
    }
}

impl Deframer for HdlcDeframer {
    fn run(&mut self) {
        while let Ok(new_bits) = self.reader.recv() {
            if new_bits.is_empty() {
                continue;
            }

            for bit in new_bits {
                self.buffer.push_back(bit);
            }

            let new_frames = self.find_frames();
            let packets = deframe(new_frames);

            self.writer.send(packets).unwrap_or_else(|e| {
                // Error should be logged somewhere
                eprintln!("Failed to send telemetry packets: {}", e);
            });
        }
    }
}

fn deframe(frames: Vec<Frame>) -> Vec<TelemetryPacket> {
    frames
        .into_iter()
        .filter_map(|frame| {
            frame
                .info
                .and_then(|info| TelemetryPacket::try_from(info).ok())
        })
        .collect()
}
