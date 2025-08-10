use crate::bitvecdeque::BitVecDeque;
use crate::frame::Frame;
use std::sync::mpsc;

// Typical HDLC frames are up to 260 bytes (2080 bits)
// 4096 bits (512 bytes) is a safe upper bound for most use cases
const MAX_BUFFER_LEN: usize = 4096;
const FLAG_ARRAY: [bool; 8] = [false, true, true, true, true, true, true, false];

pub enum ParserState {
    SearchingStartSync,
    SearchingEndSync,
}

pub struct Deframer {
    reader: mpsc::Receiver<Vec<bool>>,
    buffer: BitVecDeque,
    idx: usize, // it's an index, or offset, from the 0th element of the buffer
    state: ParserState,
}

impl Deframer {
    pub fn new(rx: mpsc::Receiver<Vec<bool>>) -> Self {
        Self {
            reader: rx,
            buffer: BitVecDeque::new(),
            idx: 0,
            state: ParserState::SearchingStartSync,
        }
    }

    // the function return is temporary for testing purposes
    pub fn run(&mut self) -> Vec<Frame> {
        let mut frames = Vec::new();
        while let Ok(new_bits) = self.reader.recv() {
            if new_bits.is_empty() {
                continue;
            }

            for bit in new_bits {
                self.buffer.push_back(bit);
            }

            let new_frames = self.find_frames();
            frames.extend(new_frames);
            // let packets = frames
            //     .into_iter()
            //     .map(|frame| Packet::new(frame))
            //     .collect::<Vec<Packet>>();
            // Publish these packets to a MQTT topic
            // Packet should be an interface so multiple packet types can be implemented
        }
        frames
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
            // Usar slice_to_bitvec para obtener 8 bits y convertir a Vec<bool>
            let bitvec_slice = self.buffer.slice_to_bitvec(self.idx, self.idx + 8);
            let slice: Vec<bool> = bitvec_slice.iter().map(|bit| *bit).collect();

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

                        if let Some(frame) = Frame::try_from(frame_bits) {
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

#[cfg(test)]
mod tests {

    use super::*;
    use std::thread;

    #[test]
    fn empty_frame_with_garbage_before_and_after() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame_bits = FLAG_ARRAY.to_vec();
            expected_frame_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], RawFrame(expected_frame_bits));
        });

        tx.send(vec![false, true, true, false]).unwrap(); // garbage
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![false, true, true, false]).unwrap(); // garbage

        drop(tx); // end signal for testing
        handle.join().unwrap();
    }

    #[test]
    fn frame_with_no_garbage() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame_bits = FLAG_ARRAY.to_vec();
            expected_frame_bits.extend_from_slice(&[true, true, true, false]);
            expected_frame_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], RawFrame(expected_frame_bits));
        });

        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![true, true, true, false]).unwrap(); // content
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn frame_with_previous_garbage() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame_bits = FLAG_ARRAY.to_vec();
            expected_frame_bits.extend_from_slice(&[true, true, true, false]);
            expected_frame_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], RawFrame(expected_frame_bits));
        });

        tx.send(vec![false, true, true, false]).unwrap(); // garbage
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![true, true, true, false]).unwrap(); // content
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn just_empty_bit_vecs() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();
            assert_eq!(frames.len(), 0);
        });

        tx.send(vec![]).unwrap();
        tx.send(vec![]).unwrap();
        tx.send(vec![]).unwrap();

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn just_garbage() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();
            assert_eq!(frames.len(), 0);
        });

        tx.send(vec![true, false, true, false]).unwrap(); // garbage
        tx.send(vec![false, true, true, false]).unwrap(); // garbage
        tx.send(vec![]).unwrap();

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn frame_with_missing_start_flag() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();
            assert_eq!(frames.len(), 0);
        });

        tx.send(vec![true, false, true, false]).unwrap(); // content
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn frame_with_missing_end_flag() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();
            assert_eq!(frames.len(), 0);
        });

        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![true, false, true, false]).unwrap(); // content

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn two_frames_with_no_garbage() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame1_bits = FLAG_ARRAY.to_vec();
            expected_frame1_bits.extend_from_slice(&[true, false, true, false]);
            expected_frame1_bits.extend_from_slice(&FLAG_ARRAY);

            let mut expected_frame2_bits = FLAG_ARRAY.to_vec();
            expected_frame2_bits.extend_from_slice(&[false, false, true, true]);
            expected_frame2_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0], RawFrame(expected_frame1_bits));
            assert_eq!(frames[1], RawFrame(expected_frame2_bits));
        });

        // Frame 1
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![true, false, true, false]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        // Frame 2
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![false, false, true, true]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn two_frames_with_garbage_between() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame1_bits = FLAG_ARRAY.to_vec();
            expected_frame1_bits.extend_from_slice(&[true, false, true, false]);
            expected_frame1_bits.extend_from_slice(&FLAG_ARRAY);

            let mut expected_frame2_bits = FLAG_ARRAY.to_vec();
            expected_frame2_bits.extend_from_slice(&[false, false, true, true]);
            expected_frame2_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0], RawFrame(expected_frame1_bits));
            assert_eq!(frames[1], RawFrame(expected_frame2_bits));
        });

        // Frame 1
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![true, false, true, false]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        // Garbage between frames
        tx.send(vec![true, true, false, false, true]).unwrap();

        // Frame 2
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![false, false, true, true]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn two_frames_with_garbage_before_between_and_after() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame1_bits = FLAG_ARRAY.to_vec();
            expected_frame1_bits.extend_from_slice(&[true, true, true, false]);
            expected_frame1_bits.extend_from_slice(&FLAG_ARRAY);

            let mut expected_frame2_bits = FLAG_ARRAY.to_vec();
            expected_frame2_bits.extend_from_slice(&[true, true, true, false]);
            expected_frame2_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0], RawFrame(expected_frame1_bits));
            assert_eq!(frames[1], RawFrame(expected_frame2_bits));
        });

        tx.send(vec![false, true, true, false]).unwrap(); // garbage

        // Frame 1
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![true, true, true, false]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        tx.send(vec![false, true, true, false]).unwrap(); // garbage

        // Frame 2
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![true, true, true, false]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        tx.send(vec![false, true, true, false]).unwrap(); // garbage

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn two_empty_frames_with_garbage_before_and_after() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame1_bits = FLAG_ARRAY.to_vec();
            expected_frame1_bits.extend_from_slice(&FLAG_ARRAY);
            let mut expected_frame2_bits = FLAG_ARRAY.to_vec();
            expected_frame2_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0], RawFrame(expected_frame1_bits));
            assert_eq!(frames[1], RawFrame(expected_frame2_bits));
        });

        tx.send(vec![false, true, true, true]).unwrap(); // garbage
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![false, true, true, false]).unwrap(); // garbage

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn flags_come_in_chunks() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame_bits = FLAG_ARRAY.to_vec();
            expected_frame_bits.extend_from_slice(&[true, true, true, false]);
            expected_frame_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], RawFrame(expected_frame_bits));
        });

        tx.send(FLAG_ARRAY[..4].to_vec()).unwrap(); // first half of sync
        tx.send(FLAG_ARRAY[4..].to_vec()).unwrap(); // second half of sync
        tx.send(vec![true, true, true, false]).unwrap(); // content
        tx.send(FLAG_ARRAY[..2].to_vec()).unwrap(); // first half of sync
        tx.send(FLAG_ARRAY[2..].to_vec()).unwrap(); // second half of sync

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn frame_with_max_buffer_length() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame_bits = FLAG_ARRAY.to_vec();
            expected_frame_bits.extend_from_slice(&[false; MAX_BUFFER_LEN - 16]);
            expected_frame_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], RawFrame(expected_frame_bits));
        });

        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        for _ in 0..MAX_BUFFER_LEN - 16 {
            // MAX_BUFFER_LEN - 16 bits of content
            tx.send(vec![false]).unwrap();
        }
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn frame_exceeds_max_buffer_length() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            assert_eq!(frames.len(), 0);
        });

        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        tx.send(vec![false; MAX_BUFFER_LEN - 15]).unwrap(); // MAX_BUFFER_LEN - 15 bits of content

        // not enough space in buffer (7 bits) for closing flag
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn frame_after_cleared_buffer() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame_bits = FLAG_ARRAY.to_vec();
            expected_frame_bits.extend_from_slice(&[true, true, true, false]);
            expected_frame_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], RawFrame(expected_frame_bits));
        });

        tx.send(vec![false; MAX_BUFFER_LEN]).unwrap();

        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![true, true, true, false]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync

        drop(tx);
        handle.join().unwrap();
    }

    #[test]
    fn frame_after_max_buffer_length_frame() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let mut deframer = Deframer::new(rx);
            let frames = deframer.run();

            let mut expected_frame1_bits = FLAG_ARRAY.to_vec();
            expected_frame1_bits.extend_from_slice(&[false; MAX_BUFFER_LEN - 16]);
            expected_frame1_bits.extend_from_slice(&FLAG_ARRAY);

            let mut expected_frame2_bits = FLAG_ARRAY.to_vec();
            expected_frame2_bits.extend_from_slice(&[true, true, true, false]);
            expected_frame2_bits.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0], RawFrame(expected_frame1_bits));
            assert_eq!(frames[1], RawFrame(expected_frame2_bits));
        });

        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        for _ in 0..MAX_BUFFER_LEN - 16 {
            tx.send(vec![false]).unwrap();
        }
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![true, true, true, false]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        drop(tx);
        handle.join().unwrap();
    }
}
