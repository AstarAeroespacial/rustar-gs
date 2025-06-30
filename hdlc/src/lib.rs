// mod my_ring_buffer;
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

pub fn deframe(rx: mpsc::Receiver<Vec<bool>>) {
    let mut buffer = BitVecDeque::new();
    let mut cursor_idx = 0;
    let mut state = ParserState::SearchingSyncStart;

    loop {
        // Agrego los nuevos bits que llegaron por el pipe.
        let new_bits = rx.recv().unwrap();
        dbg!(&new_bits);

        if new_bits.len() == 0 {
            break;
        }

        // Extender el buffer con los nuevos bits
        for bit in new_bits {
            buffer.push_back(bit);
        }
        dbg!(&buffer.to_vec());

        // Busco un sync.

        loop {
            if buffer.len() < cursor_idx + 8 {
                dbg!(buffer.len());
                break;
            }

            // Obtener slice de 8 bits usando el método slice_to_bitvec y convertir a Vec<bool>
            let bitvec_slice = buffer.slice_to_bitvec(cursor_idx, cursor_idx + 8);
            let slice: Vec<bool> = bitvec_slice.iter().map(|bit| *bit).collect();

            dbg!(&slice);

            if slice == vec![false, true, true, true, true, true, true, false] {
                dbg!("match!");
                // found sync
                match state {
                    ParserState::SearchingSyncStart => {
                        state = ParserState::SearchingSyncEnd;
                        cursor_idx += 1;
                    }
                    ParserState::SearchingSyncEnd => {
                        let frame_bits = buffer.drain_range(0, cursor_idx + 8);
                        dbg!(&frame_bits);
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
