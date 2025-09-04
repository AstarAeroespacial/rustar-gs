use crc_any::CRCu16;

pub(crate) type Bit = bool;
pub(crate) type Byte = u8;

/// Unnumbered format commands/responses.
#[derive(Debug, Clone, PartialEq)]
pub enum UnnumberedType {
    Information, // UI - Unnumbered Information
    Test,        // TEST - Unnumbered Test
}

/// Defines the function of the frame.
#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    /// The U format is used to provide additional data link control
    /// functions and unnumbered information transfer.
    Unnumbered { kind: UnnumberedType, pf: Bit },
}

#[derive(Debug)]
pub enum DeframingError {
    InvalidControlFrameType,
    InvalidControlUnnumberedType,
    InvalidFrameSize,
    InvalidPacketLength,
    PacketLengthMismatch,
    FcsMismatch,
}

const FLAG: Byte = 0b0111_1110;
const MIN_FRAME_SIZE: usize = 32; // Start flag (8) + FCS(16) + End flag (8)

/// Represents an HDLC frame.
///
/// ## Fields
/// - `info`: Optional payload data contained in the frame.
/// - `fcs`: Frame Check Sequence for error detection.
#[derive(Debug)]
pub struct Frame {
    info: Option<Vec<Byte>>,
    fcs: FrameCheckingSequence,
}

#[derive(Debug)]
struct FrameCheckingSequence(u16);

impl TryFrom<Vec<Bit>> for Frame {
    type Error = DeframingError;

    fn try_from(bits: Vec<Bit>) -> Result<Self, Self::Error> {
        if bits.len() < MIN_FRAME_SIZE {
            return Err(DeframingError::InvalidFrameSize);
        }

        // Remove flags
        let content_bits = &bits[8..bits.len() - 8];

        let content_bits = bit_destuff(content_bits);

        // Extract info bits (everything between address and FCS)
        let info_bits = if content_bits.len() < 16 {
            &[]
        } else {
            &content_bits[..content_bits.len() - 16]
        };

        // Extract FCS (last 16 bits)
        let fcs_bits = &content_bits[content_bits.len() - 16..];
        let fcs_val = bits_to_u16(fcs_bits);
        let fcs = FrameCheckingSequence(fcs_val);

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

        if let Some(b) = &info_bytes {
            let mut crc = CRCu16::crc16_x25();
            crc.digest(&b);
            let calc_fcs = crc.get_crc();

        if calc_fcs != fcs.0 {
            return Err(DeframingError::FcsMismatch);
        }
        }

        Ok(Frame {
            info: info_bytes,
            fcs,
        })
    }
}

/// Converts a slice of bits (LSB first) to a u16
fn bits_to_u16(bits: &[Bit]) -> u16 {
    bits.iter()
        .enumerate()
        .fold(0u16, |acc, (i, &b)| acc | ((b as u16) << i))
}

impl Frame {
    pub fn new(info: Option<Vec<Byte>>) -> Self {
        // 1. Preparar bytes para el CRC
        let mut data = Vec::new();

        if let Some(ref payload) = info {
            data.extend(payload);
        }

        // 2. Calcular el FCS (X-25)
        let mut crc = CRCu16::crc16_x25();
        crc.digest(&data);
        let fcs = FrameCheckingSequence(crc.get_crc());

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

/// Performs HDLC bit stuffing: After five consecutive 1s, insert a 0.
fn bit_stuff(bits_in: &[Bit]) -> Vec<Bit> {
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
fn bit_destuff(bits_in: &[Bit]) -> Vec<Bit> {
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

/// Helper to convert a Byte to a Vec<Bit>, Least Significant Bit first
fn unpack_lsb(byte: Byte) -> Vec<Bit> {
    (0..8).map(|i| (byte & (1 << i)) != 0).collect()
}

/// Packs a slice or vector of 0/1 values into u8s (LSB first ordering).
///
/// This function takes binary values (0 or 1) and packs them into bytes.
/// Each group of 8 bits becomes one u8. If the input length is not a multiple
/// of 8, the last byte will be padded with zeros in the most significant bits.
///
/// The function treats any non-zero value as 1 (true) and zero as 0 (false).
///
/// # Arguments
/// * `bits` - A slice of values that can be compared to zero
///
/// # Returns
/// A vector of u8 values containing the packed bits
///
/// # Examples
/// ```
/// let bits = [1, 0, 1, 0, 1, 0, 1, 0];
/// let bytes = pack_bits_to_bytes(&bits);
/// assert_eq!(bytes, vec![0b01010101]); // LSB first: bit 0 is rightmost
/// ```
pub fn pack_bits_to_bytes_lsb<T>(bits: &[T]) -> Vec<u8>
where
    T: Copy + PartialEq<T> + Default,
{
    bits.chunks(8)
        .map(|chunk| {
            chunk.iter().enumerate().fold(0u8, |acc, (i, &bit)| {
                let bit_val = if bit != T::default() { 1u8 } else { 0u8 };
                acc | (bit_val << i)
            })
        })
        .collect()
}

/// Packs a slice or vector of 0/1 values into u8s (MSB first ordering).
///
/// This function takes binary values (0 or 1) and packs them into bytes.
/// Each group of 8 bits becomes one u8. If the input length is not a multiple
/// of 8, the last byte will be padded with zeros in the least significant bits.
///
/// The function treats any non-zero value as 1 (true) and zero as 0 (false).
///
/// # Arguments
/// * `bits` - A slice of values that can be compared to zero
///
/// # Returns
/// A vector of u8 values containing the packed bits
///
/// # Examples
/// ```
/// let bits = [1, 0, 1, 0, 1, 0, 1, 0];
/// let bytes = pack_bits_to_bytes_msb(&bits);
/// assert_eq!(bytes, vec![0b10101010]); // MSB first: bit 0 is leftmost
/// ```
pub fn pack_bits_to_bytes_msb<T>(bits: &[T]) -> Vec<u8>
where
    T: Copy + PartialEq<T> + Default,
{
    bits.chunks(8)
        .map(|chunk| {
            chunk.iter().enumerate().fold(0u8, |acc, (i, &bit)| {
                let bit_val = if bit != T::default() { 1u8 } else { 0u8 };
                acc | (bit_val << (7 - i))
            })
        })
        .collect()
}

/// Specialized function for packing boolean slices into u8s (LSB first).
/// This is more efficient than the generic version for bool inputs.
pub fn pack_bools_to_bytes_lsb(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|chunk| {
            chunk
                .iter()
        .enumerate()
                .fold(0u8, |acc, (i, &bit)| acc | ((bit as u8) << i))
        })
        .collect()
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
        // https://crccalc.com/?crc=0&method=CRC-16/IBM-SDLC&datatype=hex&outtype=hex
        let mut crc = CRCu16::crc16_x25();
        crc.digest(b"");
        assert_eq!(crc.get_crc(), 0x00); // validación estándar
    }

    #[test]
    fn stuffing_no_ones() {
        // Sin cinco unos consecutivos, no se modifica
        let input = vec![false, false, true, false, true, false];
        let expected = input.clone();
        assert_eq!(bit_stuff(&input), expected);
    }

    #[test]
    fn stuffing_exactly_five_ones() {
        // Cinco unos consecutivos → se inserta un 0
        let input = vec![true, true, true, true, true];
        let expected = vec![true, true, true, true, true, false];
        assert_eq!(bit_stuff(&input), expected);
    }

    #[test]
    fn stuffing_five_ones_in_middle() {
        let input = vec![
            false, false, //
            true, true, true, true, true, //
            false, false,
        ];
        let expected = vec![
            false, false, //
            true, true, true, true, true, false, //
            false, false,
        ];
        assert_eq!(bit_stuff(&input), expected);
    }

    #[test]
    fn stuffing_multiple_groups() {
        // Dos grupos de cinco unos
        let input = vec![true, true, true, true, true, true, true, true, true, true];
        let expected = vec![
            true, true, true, true, true, false, true, true, true, true, true, false,
        ];
        assert_eq!(bit_stuff(&input), expected);
    }

    #[test]
    fn stuffing_with_reset() {
        // Interrupción de unos con un 0 reinicia el contador
        let input = vec![
            true, true, true, true, true, false, // stuffing tras los primeros 5
            true, true, false, // no se alcanza 5 otra vez
        ];
        let expected = vec![
            true, true, true, true, true, false, false, true, true, false,
        ];
        assert_eq!(bit_stuff(&input), expected);
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
    fn destuffed_frame_matches_original() {
        let original = vec![true, true, true, true, true, true, false, true];
        let stuffed = bit_stuff(&original);
        let destuffed = bit_destuff(&stuffed);
        assert_eq!(original, destuffed);
    }
}
