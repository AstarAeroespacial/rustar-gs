use crate::deframe::Deframer;
use hdlc::bitvecdeque::BitVecDeque;
use hdlc::frame::Frame;

/// Local parser state duplicated here so the iterator can run the same state machine.
enum ParserState {
    SearchingStartSync,
    SearchingEndSync,
}

// Typical HDLC frames are up to 260 bytes (2080 bits)
// 4096 bits (512 bytes) is a safe upper bound for most use cases
const MAX_BUFFER_LEN: usize = 4096;
const FLAG_ARRAY: [bool; 8] = [false, true, true, true, true, true, true, false];

pub struct HdlcDeframingIterator<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    input: I,
    buffer: BitVecDeque,
    idx: usize,
    state: ParserState,
}

impl<I> Iterator for HdlcDeframingIterator<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        // Keep attempting to find a frame. As soon as one is found, return it.
        loop {
            // Try to find a frame using currently buffered bits
            while self.buffer.len() >= 8 && self.buffer.len().saturating_sub(self.idx) >= 8 {
                // Drop buffer if it grows too large (prevents DoS via never-ending garbage)
                if self.buffer.len() > MAX_BUFFER_LEN {
                    self.buffer.clear();
                    self.idx = 0;
                    self.state = ParserState::SearchingStartSync;
                    break;
                }

                // Get an 8-bit slice at current idx
                let bitvec_slice = self.buffer.slice_to_bitvec(self.idx, self.idx + 8);
                let slice: Vec<bool> = bitvec_slice.iter().map(|b| *b).collect();

                if slice == FLAG_ARRAY {
                    match self.state {
                        ParserState::SearchingStartSync => {
                            self.state = ParserState::SearchingEndSync;
                            // Move past this flag and continue looking for end flag
                            self.idx = self.idx.saturating_add(8);
                        }
                        ParserState::SearchingEndSync => {
                            // Drain the whole frame between syncs (from 0 to idx+8)
                            let frame_bits = self.buffer.drain_range(0, self.idx + 8);

                            self.idx = 0;
                            self.state = ParserState::SearchingStartSync;

                            if let Ok(frame) = Frame::try_from(frame_bits) {
                                return Some(frame);
                            }
                        }
                    }
                } else {
                    match self.state {
                        ParserState::SearchingStartSync => {
                            // drop the first element; garbage before a start flag
                            self.buffer.pop_front();
                        }
                        ParserState::SearchingEndSync => {
                            // increment the index to continue looking for the closing flag
                            self.idx = self.idx.saturating_add(1);
                        }
                    }
                }
            }

            // If we cannot find a frame with current data, try to read more from input
            match self.input.next() {
                Some(new_bits) => {
                    if new_bits.is_empty() {
                        // nothing to add, continue loop to try to find a frame (or read more)
                        continue;
                    }
                    for bit in new_bits {
                        self.buffer.push_back(bit);
                    }
                    // continue the loop to attempt a new scan with added bits
                    continue;
                }
                None => {
                    // No more input; if a frame hasn't been produced, return None
                    return None;
                }
            }
        }
    }
}

pub struct HdlcDeframer<I> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I> HdlcDeframer<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I> Deframer<Vec<bool>, Frame> for HdlcDeframer<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    type Input = I;
    type Output = HdlcDeframingIterator<I>;

    fn frames(&self, input: Self::Input) -> Self::Output {
        HdlcDeframingIterator {
            input,
            buffer: BitVecDeque::new(),
            idx: 0,
            state: ParserState::SearchingStartSync,
        }
    }
}
