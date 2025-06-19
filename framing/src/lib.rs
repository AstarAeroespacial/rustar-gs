use std::io::{Read, Write, Result};
use hdlc::{FrameReader, SpecialChars};

/// Reads raw bits from a `Read` source, detects HDLC frames using `FrameReader`, and writes complete frames to a `Write` sink.
pub fn build_frames<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
) -> Result<()> {
    let mut frame_reader = FrameReader::new(&mut reader, SpecialChars::default());
    loop {
        match frame_reader.next() {
            Some(frame) => writer.write_all(&frame)?,
            None => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use hdlc::{encode, SpecialChars};

    #[test]
    fn single_frame() {
        let payload = [0x10, 0x20, 0x30];
        let input = encode(&payload, SpecialChars::default()).unwrap();
        let mut output = Vec::new();
        build_frames(Cursor::new(input.clone()), &mut output).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn two_frames() {
        let frame1 = encode(&[0xAA, 0xBB], SpecialChars::default()).unwrap();
        let frame2 = encode(&[0xCC, 0xDD, 0xEE], SpecialChars::default()).unwrap();
        let mut input = frame1;
        input.extend_from_slice(&frame2);
        let mut output = Vec::new();
        build_frames(Cursor::new(input.clone()), &mut output).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn incomplete_frame() {
        // Incomplete: missing closing flag
        let mut input = encode(&[0x99, 0x88], SpecialChars::default()).unwrap();
        input.pop(); // Remove the closing flag
        let mut output = Vec::new();
        build_frames(Cursor::new(input), &mut output).unwrap();
        // No complete frame, so output should be empty
        assert_eq!(output, Vec::<u8>::new());
    }
}
