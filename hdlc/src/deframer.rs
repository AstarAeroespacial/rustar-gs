use std::{collections::VecDeque, sync::mpsc};

struct BitVecDeque {}

impl BitVecDeque {
    /// Drains the iterator of the elements matching the predicate.
    /// Returns an iterator over the drained elements.
    fn drain_while() {}
}

enum ParserState {
    SearchingSyncStart,
    SearchingSyncEnd,
}

struct Deframer {
    reader: mpsc::Receiver<Vec<bool>>,
    buffer: VecDeque<bool>,
    idx: usize, // it's an index, or offset, from the 0th element of the buffer
    state: ParserState,
}

#[derive(Debug, PartialEq)]
struct RawDelimitedBits(Vec<bool>);

impl Deframer {
    pub fn new(rx: mpsc::Receiver<Vec<bool>>) -> Self {
        Self {
            reader: rx,
            buffer: VecDeque::new(),
            idx: 0,
            state: ParserState::SearchingSyncStart,
        }
    }

    fn run(&mut self) {
        while let Ok(new_bits) = self.reader.recv() {
            self.buffer.extend(new_bits);

            let raw_delimited_frames = self.try_find_delimited();
        }
    }

    fn try_find_delimited(&mut self) -> Vec<RawDelimitedBits> {
        let mut frames = Vec::new();

        // TODO: add a MAX, because if i never find an ending sync the while never ends, and drop the buffer contents
        while self.buffer.len() - self.idx >= 8 {
            let slice = self
                .buffer
                .range(self.idx..(self.idx + 8))
                .copied()
                .collect::<Vec<_>>(); // try to avoid this copy

            // if i found sync 01111110
            if slice == [false, true, true, true, true, true, true, false] {
                match self.state {
                    // if i was looking for the beginning of a frame
                    ParserState::SearchingSyncStart => {
                        // i found it, so i update the state
                        self.state = ParserState::SearchingSyncEnd;
                        // i update the index, so i begin looking for the end sync
                        self.idx += 1; // i could advance it by 8 actually, to fast forward the sync
                    }
                    // if i was looking for the end of the frame
                    ParserState::SearchingSyncEnd => {
                        // i drain the whole frame, between syncs
                        let frame_bits = self.buffer.drain(..(self.idx + 8)).collect::<Vec<_>>();
                        frames.push(RawDelimitedBits(frame_bits));
                        // and reset the index to 0 and the parser state, so i can begin again
                        self.idx = 0;
                        self.state = ParserState::SearchingSyncStart
                    }
                }
            }
            // if i didn't find a sync
            else {
                match self.state {
                    // and if i'm looking for the starting sync
                    ParserState::SearchingSyncStart => {
                        // i drop the first element, it will be lost, but alas, such is life...
                        // this is something to try to avoid
                        self.buffer.pop_front();
                    }
                    // and if i'm looking for ending sync
                    ParserState::SearchingSyncEnd => {
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

    use std::thread;

    use super::*;

    #[test]
    fn one_frame() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let mut deframer = Deframer::new(rx);

        deframer.buffer = vec![
            false, true, true, false, // garbage
            false, true, true, true, true, true, true, false, // sync
            true, true, true, false, // content
            false, true, true, true, true, true, true, false, // sync
            false, true, true, false, // garbage
        ]
        .into();

        let frames = deframer.try_find_delimited();

        assert_eq!(
            frames,
            vec![RawDelimitedBits(vec![
                false, true, true, true, true, true, true, false, // sync
                true, true, true, false, // content
                false, true, true, true, true, true, true, false, // sync
            ])]
        )
    }

    #[test]
    fn empty_frame() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let mut deframer = Deframer::new(rx);

        deframer.buffer = vec![
            false, true, true, false, // garbage
            false, true, true, true, true, true, true, false, // sync
            false, true, true, true, true, true, true, false, // sync
            false, true, true, false, // garbage
        ]
        .into();

        let frames = deframer.try_find_delimited();

        assert_eq!(
            frames,
            vec![RawDelimitedBits(vec![
                false, true, true, true, true, true, true, false, // sync
                false, true, true, true, true, true, true, false, // sync
            ])]
        )
    }

    #[test]
    fn two_frames() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let mut deframer = Deframer::new(rx);

        deframer.buffer = vec![
            false, true, true, false, // garbage
            false, true, true, true, true, true, true, false, // sync
            true, true, true, false, // content
            false, true, true, true, true, true, true, false, // sync
            false, true, true, false, // garbage
            false, true, true, true, true, true, true, false, // sync
            true, true, true, false, // content
            false, true, true, true, true, true, true, false, // sync
            false, true, true, false, // garbage
        ]
        .into();

        let frames = deframer.try_find_delimited();

        assert_eq!(
            frames,
            vec![
                RawDelimitedBits(vec![
                    false, true, true, true, true, true, true, false, // sync
                    true, true, true, false, // content
                    false, true, true, true, true, true, true, false, // sync
                ]),
                RawDelimitedBits(vec![
                    false, true, true, true, true, true, true, false, // sync
                    true, true, true, false, // content
                    false, true, true, true, true, true, true, false, // sync
                ])
            ]
        )
    }
}
