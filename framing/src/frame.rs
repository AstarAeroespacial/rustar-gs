use crc_any::CRCu16;

pub(crate) type Bit = bool;
pub(crate) type Byte = u8;

#[derive(Debug)]
pub enum DeframingError {
    InvalidFrameSize,
    InvalidPacketLength,
    PacketLengthMismatch,
    FcsMismatch,
}

const FLAG: Byte = 0b0111_1110;
const MIN_FRAME_SIZE: usize = 32; // Start flag (8) + empty Info(0) + FCS(16) + End flag (8)

/// Represents an HDLC frame.
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub info: Option<Vec<Byte>>,
    fcs: FrameCheckingSequence,
}

#[derive(Debug, PartialEq, Eq)]
struct FrameCheckingSequence(u16);

impl TryFrom<Vec<Bit>> for Frame {
    type Error = DeframingError;

    fn try_from(bits: Vec<Bit>) -> Result<Self, Self::Error> {
        if bits.len() < MIN_FRAME_SIZE {
            return Err(DeframingError::InvalidFrameSize);
        }

        // Verify start and end flags
        let start_flag = &bits[0..8];
        let end_flag = &bits[bits.len() - 8..];
        let expected_flag = unpack_lsb(FLAG);

        if start_flag != expected_flag || end_flag != expected_flag {
            return Err(DeframingError::InvalidFrameSize);
        }

        // Remove flags
        let content_bits = &bits[8..bits.len() - 8];

        let content_bits = bit_destuff(content_bits);

        if content_bits.len() < 16 {
            return Err(DeframingError::InvalidFrameSize);
        }

        // Extract info bits (everything between the first flag and FCS)
        let info_bits = &content_bits[..content_bits.len() - 16];

        // Extract FCS (last 16 bits)
        let fcs_bits = &content_bits[content_bits.len() - 16..];
        let received_fcs = bits_to_u16(fcs_bits);
        let fcs = FrameCheckingSequence(received_fcs);

        // Convert info_bits to bytes
        let info_bytes = if info_bits.is_empty() {
            None
        } else {
            let mut bytes = Vec::new();
            for chunk in info_bits.chunks(8) {
                let mut byte = 0u8;
                for (i, &b) in chunk.iter().enumerate() {
                    byte |= (b as u8) << i;
                }
                bytes.push(byte);
            }
            Some(bytes)
        };

        let calc_fcs = calculate_fcs(&info_bytes);
        if calc_fcs != fcs.0 {
            return Err(DeframingError::FcsMismatch);
        }

        Ok(Frame {
            info: info_bytes,
            fcs,
        })
    }
}

impl Frame {
    pub fn new(info: Option<Vec<Byte>>) -> Self {
        // 1. Preparar bytes para el CRC
        let mut data: Vec<u8> = Vec::new();

        if let Some(ref payload) = info {
            data.extend(payload);
        }

        // 2. Calcular el FCS
        let crc = calculate_fcs(&info);
        let fcs = FrameCheckingSequence(crc);

        // 3. Crear el frame
        Frame { info, fcs }
    }

    /// Converts a Frame into a vector of bits.
    pub fn to_bits(&self) -> Vec<Bit> {
        let mut raw_bits = Vec::new();

        if let Some(info) = &self.info {
            for byte in info {
                raw_bits.extend(unpack_lsb(*byte));
            }
        }

        // Append CRC, in little endian. Not ISO compliant, but it's what GNU radio's HDLC framer does.
        // https://github.com/gnuradio/gnuradio/blob/721e477cdb4ed22214ed886d6063cff2dac7d0b5/gr-digital/lib/hdlc_framer_pb_impl.cc#L133
        let crc_bytes = self.fcs.0.to_le_bytes();
        raw_bits.extend(unpack_lsb(crc_bytes[0]));
        raw_bits.extend(unpack_lsb(crc_bytes[1]));

        // Apply bit stuffing to the entire content between flags
        let stuffed_bits = bit_stuff(&raw_bits);

        // Build frame with flags
        let mut bits = Vec::new();
        bits.extend(unpack_lsb(FLAG));
        bits.extend(stuffed_bits);
        bits.extend(unpack_lsb(FLAG));

        bits
    }
}

/// Helper to convert a Byte to a Vec<Bit>, Least Significant Bit first
fn unpack_lsb(byte: Byte) -> Vec<Bit> {
    (0..8).map(|i| (byte & (1 << i)) != 0).collect()
}

/// Performs HDLC bit stuffing: After five consecutive 1s, insert a 0.
pub fn bit_stuff(bits_in: &[Bit]) -> Vec<Bit> {
    let mut stuffed = Vec::new();
    let mut ones_count = 0;

    for &bit in bits_in {
        stuffed.push(bit);
        if bit {
            ones_count += 1;
            if ones_count == 5 {
                stuffed.push(false); // insert a 0
                ones_count = 0;
            }
        } else {
            ones_count = 0;
        }
    }
    stuffed
}

/// Performs HDLC bit destuffing: After five consecutive 1s, remove the following 0 if present.
pub fn bit_destuff(bits_in: &[Bit]) -> Vec<Bit> {
    let mut destuffed = Vec::new();
    let mut ones_count = 0;
    let mut i = 0;

    while i < bits_in.len() {
        let bit = bits_in[i];
        destuffed.push(bit);

        if bit {
            ones_count += 1;
            if ones_count == 5 && i + 1 < bits_in.len() {
                // After 5 ones, if next bit is 0, skip it (was stuffed)
                if !bits_in[i + 1] {
                    i += 1;
                }
                ones_count = 0;
            }
        } else {
            ones_count = 0;
        }
        i += 1;
    }
    destuffed
}

/// Calculates the FCS for the given info bytes.
fn calculate_fcs(info_bytes: &Option<Vec<Byte>>) -> u16 {
    let mut data = Vec::new();

    if let Some(payload) = info_bytes {
        data.extend(payload);
    }

    let mut crc = CRCu16::crc16_x25();
    crc.digest(&data);
    crc.get_crc()
}

/// Converts a slice of bits (LSB first) to a u16
fn bits_to_u16(bits: &[Bit]) -> u16 {
    bits.iter()
        .enumerate()
        .fold(0u16, |acc, (i, &b)| acc | ((b as u16) << i))
}

/// Specialized function for packing boolean slices into u8s (MSB first).
/// This is more efficient than the generic version for bool inputs.
pub fn pack_bools_to_bytes_msb(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0u8, |acc, (i, &bit)| acc | ((bit as u8) << (7 - i)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fcs_on_known_data() {
        // https://crccalc.com/?crc=123456789&method=CRC-16/IBM-SDLC&datatype=hex&outtype=hex
        let mut crc = CRCu16::crc16_x25();
        crc.digest(&[0x12, 0x34, 0x56, 0x78, 0x09]);
        assert_eq!(crc.get_crc(), 0xA55E); // validación estándar
    }

    #[test]
    fn fcs_on_empty() {
        // https://crccalc.com/?crc=&method=CRC-16/IBM-SDLC&datatype=hex&outtype=hex
        let mut crc = CRCu16::crc16_x25();
        crc.digest(b"");
        assert_eq!(crc.get_crc(), 0x00); // validación estándar
    }

    #[test]
    fn frame_empty_payload() {
        let bits = Frame::new(None).to_bits();
        let packed = pack_bools_to_bytes_msb(&bits);

        let expected = vec![0x7e_u8, 0x00, 0x00, 0x7e];
        assert_eq!(packed, expected);
    }

    #[test]
    fn frame_some_payload() {
        let bits = Frame::new(Some("HOLA FRANK".as_bytes().to_vec())).to_bits();
        let packed = pack_bools_to_bytes_msb(&bits);

        let expected = vec![
            0x7e_u8, 0x12, 0xf2, 0x32, 0x82, 0x04, 0x62, 0x4a, 0x82, 0x72, 0xd2, 0x09, 0x43, 0x7e,
        ];
        assert_eq!(packed, expected);
    }

    #[test]
    fn stuffing_no_ones() {
        // Sin cinco unos consecutivos, no se modifica
        let input = vec![false, false, true, false, true, false];
        let expected = input.clone();
        assert_eq!(bit_stuff(&input), expected);
    }
}
