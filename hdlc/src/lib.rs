mod bitvecdeque;
mod deframer;

use bitvecdeque::BitVecDeque;
use deframer::ParserState;
use std::sync::mpsc;

// Esta es una implementación bastante naive, TODO:
// 1. Los bool ocupan u8. Deberían recibirse u8s a interpretar como bits packeados.
// 2. Buscar cómo evitar copies.
// 3. Tendría que ser un struct.
// 4. Async.
// 5. Pushback o algo, sino el ringbuffer podría crecer indiscriminadamente.

const FLAG_ARRAY: [bool; 8] = [false, true, true, true, true, true, true, false];

pub fn read_frames(rx: mpsc::Receiver<Vec<bool>>) -> Vec<Vec<bool>> {
    let mut buffer = BitVecDeque::new();
    let mut cursor_idx = 0;
    let mut state = ParserState::SearchingStartSync;
    let mut frames: Vec<Vec<bool>> = Vec::new(); // TODO: this should be a Vec<Frame>

    loop {
        // Agrego los nuevos bits que llegaron por el pipe
        let new_bits = rx.recv().unwrap();
        dbg!(&new_bits);

        if new_bits.is_empty() {
            break;
        }

        // Extender el buffer con los nuevos bits
        for bit in new_bits {
            buffer.push_back(bit);
        }
        dbg!(&buffer.to_vec());

        // Busco un sync
        loop {
            if buffer.len() < cursor_idx + 8 {
                dbg!(buffer.len());
                break;
            }

            // Obtener slice de 8 bits usando el método slice_to_bitvec y convertir a Vec<bool>
            let bitvec_slice = buffer.slice_to_bitvec(cursor_idx, cursor_idx + 8);
            let slice: Vec<bool> = bitvec_slice.iter().map(|bit| *bit).collect();

            dbg!(&slice);

            if slice == FLAG_ARRAY {
                dbg!("found sync");
                // found sync
                match state {
                    ParserState::SearchingStartSync => {
                        state = ParserState::SearchingEndSync;
                        cursor_idx += 8;
                    }
                    ParserState::SearchingEndSync => {
                        let frame_bits = buffer.drain_range(0, cursor_idx + 8);
                        dbg!(&frame_bits);
                        // if let Some(frame) = Frame::new(frame_bits);
                        //     frames.push(frame);
                        frames.push(frame_bits);

                        state = ParserState::SearchingStartSync;
                        cursor_idx = 0;
                    }
                }
            } else {
                match state {
                    ParserState::SearchingStartSync => {
                        buffer.pop_front();
                    }
                    ParserState::SearchingEndSync => {
                        cursor_idx += 1;
                    }
                }
            }
        }
    }
    frames
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::thread;

    #[test]
    fn basic_frame_with_no_garbage() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let frames = read_frames(rx);
            let mut expected_frame = FLAG_ARRAY.to_vec();
            expected_frame.extend_from_slice(&[true, true, true, false]);
            expected_frame.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], expected_frame);
        });

        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![true, true, true, false]).unwrap(); // content
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![]).unwrap();

        handle.join().unwrap();
    }

    #[test]
    fn basic_frame_with_previous_garbage() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let frames = read_frames(rx);
            let mut expected_frame = FLAG_ARRAY.to_vec();
            expected_frame.extend_from_slice(&[true, true, true, false]);
            expected_frame.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], expected_frame);
        });

        tx.send(vec![false, true, true, false]).unwrap(); // garbage
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![true, true, true, false]).unwrap(); // content
        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![]).unwrap();

        handle.join().unwrap();
    }

    #[test]
    fn frame_with_missing_end_flag() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let frames = read_frames(rx);
            assert_eq!(frames.len(), 0);
        });

        tx.send(FLAG_ARRAY.to_vec()).unwrap(); // sync
        tx.send(vec![true, false, true, false]).unwrap(); // content
        tx.send(vec![]).unwrap();

        handle.join().unwrap();
    }

    #[test]
    fn two_frames_with_no_garbage() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let frames = read_frames(rx);

            let mut expected_frame1 = FLAG_ARRAY.to_vec();
            expected_frame1.extend_from_slice(&[true, false, true, false]);
            expected_frame1.extend_from_slice(&FLAG_ARRAY);

            let mut expected_frame2 = FLAG_ARRAY.to_vec();
            expected_frame2.extend_from_slice(&[false, false, true, true]);
            expected_frame2.extend_from_slice(&FLAG_ARRAY);

            dbg!(&frames);
            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0], expected_frame1);
            assert_eq!(frames[1], expected_frame2);
        });

        // Frame 1
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![true, false, true, false]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        // Frame 2
        tx.send(FLAG_ARRAY.to_vec()).unwrap();
        tx.send(vec![false, false, true, true]).unwrap();
        tx.send(FLAG_ARRAY.to_vec()).unwrap();

        tx.send(vec![]).unwrap();

        handle.join().unwrap();
    }

    #[test]
    fn two_frames_with_garbage_between() {
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            let frames = read_frames(rx);

            let mut expected_frame1 = FLAG_ARRAY.to_vec();
            expected_frame1.extend_from_slice(&[true, false, true, false]);
            expected_frame1.extend_from_slice(&FLAG_ARRAY);

            let mut expected_frame2 = FLAG_ARRAY.to_vec();
            expected_frame2.extend_from_slice(&[false, false, true, true]);
            expected_frame2.extend_from_slice(&FLAG_ARRAY);

            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0], expected_frame1);
            assert_eq!(frames[1], expected_frame2);
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

        tx.send(vec![]).unwrap();

        handle.join().unwrap();
    }
}
