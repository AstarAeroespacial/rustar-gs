use crate::bitvecdeque::BitVecDeque;
use crate::deframe::Deframer;
use crate::frame::Frame;

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

impl<I> Default for HdlcDeframer<I> {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helpers for tests -------------------------------------------------
    fn frame_bits_with_info(info: Option<Vec<u8>>) -> Vec<bool> {
        // Use a standard address and control for tests
        let address = 0xFFu8;
        let control = crate::frame::Control::Unnumbered {
            kind: crate::frame::UnnumberedType::Information,
            pf: false,
        };
        let frame = Frame::new(address, control, info);
        frame.to_bits()
    }

    fn split_bits(bits: &[bool], split_at: usize) -> (Vec<bool>, Vec<bool>) {
        let a = bits[..split_at].to_vec();
        let b = bits[split_at..].to_vec();
        (a, b)
    }
    // End helpers for tests -------------------------------------------------

    #[test]
    fn test_empty_input() {
        let deframer = HdlcDeframer::new();
        let input: Vec<Vec<bool>> = vec![vec![], vec![], vec![]];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_just_garbage() {
        let deframer = HdlcDeframer::new();
        let input = vec![
            vec![true, false, true, false],
            vec![false, true, true, false],
            vec![],
        ];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_frame_with_no_garbage() {
        let deframer = HdlcDeframer::new();
        let bits = frame_bits_with_info(Some(vec![0x0F]));
        // send the whole frame as three chunks: flag, payload, flag (simulates small pushes)
        let (first, rest) = split_bits(&bits, 8);
        let (middle, last) = split_bits(&rest, rest.len() - 8);

        let input = vec![first, middle, last];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].to_bits(), bits);
    }

    #[test]
    fn test_frame_with_previous_garbage() {
        let deframer = HdlcDeframer::new();
        let bits = frame_bits_with_info(Some(vec![0x0F]));
        let mut input = vec![vec![false, true, true, false]]; // garbage
        input.push(bits.clone());
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].to_bits(), bits);
    }

    #[test]
    fn test_frame_with_missing_start_flag() {
        let deframer = HdlcDeframer::new();
        let bits = frame_bits_with_info(Some(vec![0x0F]));
        // strip the starting flag (first 8 bits)
        let truncated = bits[8..].to_vec();
        let input = vec![truncated];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        // Without a starting flag there should be no frames
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_frame_with_missing_end_flag() {
        let deframer = HdlcDeframer::new();
        let bits = frame_bits_with_info(Some(vec![0x0F]));
        // strip the ending flag (last 8 bits)
        let truncated = bits[..bits.len() - 8].to_vec();
        let input = vec![truncated];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        // Without an ending flag there should be no frames
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_two_frames_with_no_garbage() {
        let deframer = HdlcDeframer::new();
        let bits1 = frame_bits_with_info(Some(vec![0x01]));
        let bits2 = frame_bits_with_info(Some(vec![0x02]));
        let input = vec![bits1.clone(), bits2.clone()];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].to_bits(), bits1);
        assert_eq!(frames[1].to_bits(), bits2);
    }

    #[test]
    fn test_two_frames_with_garbage_between() {
        let deframer = HdlcDeframer::new();
        let bits1 = frame_bits_with_info(Some(vec![0x01]));
        let bits2 = frame_bits_with_info(Some(vec![0x02]));
        let input = vec![
            bits1.clone(),
            vec![true, true, false, false, true], // garbage
            bits2.clone(),
        ];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].to_bits(), bits1);
        assert_eq!(frames[1].to_bits(), bits2);
    }

    #[test]
    fn test_two_empty_frames_with_garbage_before_and_after() {
        let deframer = HdlcDeframer::new();
        // create two frames with empty info (valid minimal frames)
        let bits1 = frame_bits_with_info(None);
        let bits2 = frame_bits_with_info(None);
        let input = vec![
            vec![false, true, true, true],
            bits1.clone(),
            bits2.clone(),
            vec![false, true, true, false],
        ];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].to_bits(), bits1);
        assert_eq!(frames[1].to_bits(), bits2);
    }

    #[test]
    fn test_flags_come_in_chunks() {
        let deframer = HdlcDeframer::new();
        let bits = frame_bits_with_info(Some(vec![0x0F]));
        // split the starting flag into two halves
        let (first4, rest) = split_bits(&bits, 4);
        let (middle, last4) = split_bits(&rest, rest.len() - 4);
        let input = vec![first4, middle, last4];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].to_bits(), bits);
    }

    #[test]
    fn test_destuffed_frame_matches_original_helper() {
        // Sanity test: bit stuffing/destuffing works as expected
        let original = vec![true, true, true, true, true, true, false, true];
        let stuffed = crate::frame::bit_stuff(&original);
        let destuffed = crate::frame::bit_destuff(&stuffed);
        assert_eq!(original, destuffed);
    }

    #[test]
    fn test_empty_frame_with_garbage_before_and_after() {
        let deframer = HdlcDeframer::new();
        // garbage, a minimal empty frame, garbage
        let empty_bits = frame_bits_with_info(None);
        let input = vec![
            vec![false, true, true, false],
            empty_bits.clone(),
            vec![false, true, true, false],
        ];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].to_bits(), empty_bits);
    }

    #[test]
    fn test_two_frames_with_garbage_before_between_and_after() {
        let deframer = HdlcDeframer::new();
        let bits1 = frame_bits_with_info(Some(vec![0x0A]));
        let bits2 = frame_bits_with_info(Some(vec![0x0B]));
        let input = vec![
            vec![false, true, true, false],
            bits1.clone(),
            vec![true, true, false],
            bits2.clone(),
            vec![false, true, true, false],
        ];
        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].to_bits(), bits1);
        assert_eq!(frames[1].to_bits(), bits2);
    }

    #[test]
    fn test_frame_exceeds_max_buffer_length() {
        let deframer = HdlcDeframer::new();
        // Start with a flag, then flood buffer with single-bit chunks until overflow, then send closing flag
        let mut input: Vec<Vec<bool>> = Vec::new();
        input.push(FLAG_ARRAY.to_vec());
        for _ in 0..(MAX_BUFFER_LEN - 15) {
            input.push(vec![false]);
        }
        input.push(FLAG_ARRAY.to_vec());

        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        // buffer overflow should clear and no valid frames should be produced
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_frame_after_cleared_buffer() {
        let deframer = HdlcDeframer::new();
        // First send a huge block to trigger buffer clear, then send a valid frame
        let mut input: Vec<Vec<bool>> = Vec::new();
        input.push(vec![false; MAX_BUFFER_LEN]);
        let valid = frame_bits_with_info(Some(vec![0x0F]));
        input.push(valid.clone());

        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].to_bits(), valid);
    }

    #[test]
    fn test_frame_after_max_buffer_length_frame() {
        let deframer = HdlcDeframer::new();
        // Simulate a very large (invalid) frame that overflows buffer and then a normal frame
        let mut input: Vec<Vec<bool>> = Vec::new();
        input.push(FLAG_ARRAY.to_vec());
        for _ in 0..(MAX_BUFFER_LEN - 16) {
            input.push(vec![false]);
        }
        input.push(FLAG_ARRAY.to_vec());

        // Now a normal valid frame
        let valid = frame_bits_with_info(Some(vec![0x11]));
        input.push(valid.clone());

        let frames: Vec<Frame> = deframer.frames(input.into_iter()).collect();
        // The oversized/invalid frame is dropped by buffer-clearing logic; the following valid frame should be parsed
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].to_bits(), valid);
    }
}
