use hdlc::{FrameReader, SpecialChars};
use std::io::{Read, Result, Write};

/// Reads raw bits from a `Read` source, detects HDLC frames using `FrameReader`, and writes complete frames to a `Write` sink.
pub fn build_frames<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<()> {
    let frame_reader = FrameReader::new(&mut reader, SpecialChars::default());
    for frame in frame_reader {
        writer.write_all(&frame)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hdlc::{SpecialChars, encode};
    use std::io::Cursor;

    #[test]
    fn encode_basic_payload() {
        let payload = [0x01, 0x02, 0x03];
        let encoded = encode(&payload, SpecialChars::default()).unwrap();

        assert_eq!(encoded, &[0x7E, 0x01, 0x02, 0x03, 0x7E]); // FEND, payload, FEND
    }

    #[test]
    fn encode_payload_with_special_chars() {
        let payload = [0x7E, 0x7D, 0x01];
        let encoded = encode(&payload, SpecialChars::default()).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            encoded,
            &[
                0x7E, // FEND
                0x7D, // Escape byte for 0x7E (starts escape)
                0x5E, // Escaped 0x7E
                0x7D, // Escape byte for 0x7E (ends escape)
                0x5D, // Trade byte for 0x7D
                0x01,
                0x7E, // FEND
            ]
        );
    }

    #[test]
    fn build_single_frame() {
        let payload = [0x10, 0x20, 0x30];
        let frame = encode(&payload, SpecialChars::default()).unwrap();
        let mut output = Vec::new();

        build_frames(Cursor::new(frame.clone()), &mut output).unwrap();
        assert_eq!(output, frame);
    }

    #[test]
    fn build_two_frames() {
        let frame1 = encode(&[0xAA, 0xBB], SpecialChars::default()).unwrap();
        let frame2 = encode(&[0xCC, 0xDD, 0xEE], SpecialChars::default()).unwrap();
        let mut input = frame1;
        input.extend_from_slice(&frame2);
        let mut output = Vec::new();

        build_frames(Cursor::new(input.clone()), &mut output).unwrap();
        assert_eq!(output, input);
        println!("Output: {:?}", output);
    }

    #[test]
    fn build_incomplete_frame() {
        let mut frame = encode(&[0x99, 0x88], SpecialChars::default()).unwrap();
        frame.pop(); // Remove the closing flag
        let mut output = Vec::new();
        
        build_frames(Cursor::new(frame), &mut output).unwrap();
        // No complete frame, so output should be empty
        assert_eq!(output, Vec::<u8>::new());
    }
}
