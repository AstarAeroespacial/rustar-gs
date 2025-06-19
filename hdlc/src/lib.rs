// mod my_ring_buffer;
mod deframer;
use std::{collections::VecDeque, sync::mpsc};

// Esta es una implementación bastante naive:
// 1. Los bool ocupan u8. Deberían recibirse u8, a interpretar com bits packeados.
// 2. Buscar cómo evitar copies.
// 3. Tendría que ser un struct.
// 4. Async.
// 5. Pushback o algo, sino el ringbuffer podría crecer indiscriminadamente.

enum ParserState {
    SearchingSyncStart,
    SearchingSyncEnd,
}

pub fn deframe(rx: mpsc::Receiver<Vec<bool>>) {
    let mut buffer = VecDeque::<bool>::new();

    let mut cursor_idx = 0;

    let mut state = ParserState::SearchingSyncStart;

    loop {
        // Agrego los nuevos bits que llegaron por el pipe.
        let new_bits = rx.recv().unwrap();
        dbg!(&new_bits);

        if new_bits.len() == 0 {
            break;
        }

        buffer.extend(new_bits);
        dbg!(&buffer);

        // Busco un sync.

        loop {
            if buffer.len() < cursor_idx + 8 {
                dbg!(buffer.len());
                break;
            }

            let slice = buffer
                .range(cursor_idx..(cursor_idx + 8))
                .copied()
                .collect::<Vec<_>>(); // try to avoid this copy

            dbg!(&slice);

            if slice == [false, true, true, true, true, true, true, false] {
                dbg!("match!");
                // found sync
                match state {
                    ParserState::SearchingSyncStart => {
                        state = ParserState::SearchingSyncEnd;
                        cursor_idx += 1;
                    }
                    ParserState::SearchingSyncEnd => {
                        let frame_bits = buffer.drain(..(cursor_idx + 8)).collect::<Vec<_>>();
                        dbg!(frame_bits);
                        cursor_idx = 0;
                    }
                }
            } else {
                match state {
                    ParserState::SearchingSyncStart => {
                        buffer.pop_front();
                    }
                    ParserState::SearchingSyncEnd => {
                        cursor_idx += 1;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::thread;

    use super::*;

    #[test]
    fn it_works() {
        dbg!("hola");
        let (tx, rx) = mpsc::channel::<Vec<bool>>();

        let handle = thread::spawn(move || {
            dbg!("hola");
            deframe(rx);
        });

        tx.send(vec![false, true, true, false]).unwrap();
        tx.send(vec![false, true, true, true]).unwrap();
        tx.send(vec![true, true, true, false]).unwrap();
        tx.send(vec![true, true, true, false]).unwrap(); // contenido
        tx.send(vec![false, true, true, true]).unwrap();
        tx.send(vec![true, true, true, false]).unwrap();
        tx.send(vec![]).unwrap();

        handle.join().unwrap();
    }
}
