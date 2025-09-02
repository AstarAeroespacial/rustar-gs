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

impl TryFrom<Byte> for Control {
    type Error = DeframingError;

    fn try_from(value: Byte) -> Result<Self, Self::Error> {
        if (value & 0b1100_0000) >> 6 != 0b11 {
            dbg!("Only U frames supported!");
            return Err(DeframingError::InvalidControlFrameType);
        }

        // Mask the M bits.
        let modifier_bits = value & 0b11_0111;

        let try_kind = match modifier_bits {
            0b00_0000 => Ok(UnnumberedType::Information),
            0b00_0111 => Ok(UnnumberedType::Test),
            _ => {
                dbg!("Only UI and TEST unnumbered frames supported!");
                Err(DeframingError::InvalidControlUnnumberedType)
            }
        };

        match try_kind {
            Ok(kind) => {
                let pf = (value & 0b1000) >> 3 == 1;

                Ok(Self::Unnumbered { kind, pf })
            }
            Err(err) => Err(err),
        }
    }
}

impl From<Control> for Byte {
    fn from(value: Control) -> Self {
        match value {
            Control::Unnumbered { kind, pf } => {
                let byte = 0b1100_0000;

                let kind_bits = match kind {
                    UnnumberedType::Information => 0b00_0000,
                    UnnumberedType::Test => 0b00_0111,
                };

                let pf_bit = if pf { 0b1000 } else { 0b0000 };

                byte | kind_bits | pf_bit
            }
        }
    }
}

const FLAG: Byte = 0b0111_1110;
const MIN_FRAME_SIZE: usize = 48; // Start flag (8) + Address(8) + Control(8) + empty Info(0) + FCS(16) + End flag (8)

/// Represents an HDLC frame.
///
/// ## Fields
/// - `address`: The address field of the frame.
/// - `control`: The control field, representing frame's function.
/// - `info`: Optional payload data contained in the frame.
/// - `fcs`: Frame Check Sequence for error detection.
#[derive(Debug)]
pub struct Frame {
    address: Byte,
    pub control: Control,
    pub info: Option<Vec<Byte>>,
    fcs: FrameCheckingSequence,
}

#[derive(Debug)]
struct FrameCheckingSequence(u16);

impl FrameCheckingSequence {
    /// Converts a FrameCheckingSequence into a vector of bits.
    pub fn to_bits(&self) -> Vec<Bit> {
        (0..16).map(|i| (self.0 & (1 << i)) != 0).collect()
    }
}

impl TryFrom<Vec<Bit>> for Frame {
    type Error = DeframingError;

    fn try_from(bits: Vec<Bit>) -> Result<Self, Self::Error> {
        if bits.len() < MIN_FRAME_SIZE {
            return Err(DeframingError::InvalidFrameSize);
        }

        // Remove flags
        let content_bits = &bits[8..bits.len() - 8];
        let mut idx = 0;

        let content_bits = bit_destuff(content_bits);

        // Extract address
        let address_bits = &content_bits[idx..idx + 8];
        let address = bits_to_byte(address_bits);
        idx += 8;

        // Extract control
        let control_bits = &content_bits[idx..idx + 8];
        let control_byte = bits_to_byte(control_bits);
        let control = Control::try_from(control_byte)?;
        idx += 8;

        // Extract info bits (everything between address and FCS)
        let info_bits = if content_bits.len() < idx + 16 {
            &[]
        } else {
            &content_bits[idx..content_bits.len() - 16]
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

        let calc_fcs = calculate_fcs(address, control_byte, &info_bytes);
        if calc_fcs != fcs.0 {
            return Err(DeframingError::FcsMismatch);
        }

        Ok(Frame {
            address,
            control,
            info: info_bytes,
            fcs,
        })
    }
}

impl Frame {
    pub fn new(address: Byte, control: Control, info: Option<Vec<Byte>>) -> Self {
        // 1. Preparar bytes para el CRC
        let mut data = Vec::new();
        data.push(address);
        data.push(control.clone().into());

        if let Some(ref payload) = info {
            data.extend(payload);
        }

        // 2. Calcular el FCS (CRC-16-CCITT-FALSE)
        let mut crc = CRCu16::crc16ccitt_false();
        crc.digest(&data);
        let fcs = FrameCheckingSequence(crc.get_crc());

        // 3. Crear el frame
        Frame {
            address,
            control,
            info,
            fcs,
        }
    }

    /// Converts a Frame into a vector of bits.
    pub fn to_bits(&self) -> Vec<Bit> {
        let mut raw_bits = Vec::new();

        // Add fields without bit stuffing
        raw_bits.extend(byte_to_bits(self.address));
        raw_bits.extend(byte_to_bits(self.control.clone().into()));
        if let Some(info) = &self.info {
            for byte in info {
                raw_bits.extend(byte_to_bits(*byte));
            }
        }
        raw_bits.extend(self.fcs.to_bits());

        // Apply bit stuffing to the entire content between flags
        let stuffed_bits = bit_stuff(&raw_bits);

        // Build frame with flags
        let mut bits = Vec::new();
        bits.extend(byte_to_bits(FLAG));
        bits.extend(stuffed_bits);
        bits.extend(byte_to_bits(FLAG));

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

/// Calculates the FCS (CRC-16-CCITT-FALSE) for the given address, control, and info bytes.
fn calculate_fcs(address: Byte, control_byte: Byte, info_bytes: &Option<Vec<Byte>>) -> u16 {
    let mut data = Vec::new();
    data.push(address);
    data.push(control_byte);
    if let Some(payload) = info_bytes {
        data.extend(payload);
    }
    let mut crc = CRCu16::crc16ccitt_false();
    crc.digest(&data);
    crc.get_crc()
}

/// Helper to convert a Byte to a Vec<Bit>, Least Significant Bit first
fn byte_to_bits(byte: Byte) -> Vec<Bit> {
    (0..8).map(|i| (byte & (1 << i)) != 0).collect()
}

/// Converts a slice of bits (LSB first) to a byte
fn bits_to_byte(bits: &[Bit]) -> u8 {
    bits.iter()
        .enumerate()
        .fold(0u8, |acc, (i, &b)| acc | ((b as u8) << i))
}

/// Converts a slice of bits (LSB first) to a u16
fn bits_to_u16(bits: &[Bit]) -> u16 {
    bits.iter()
        .enumerate()
        .fold(0u16, |acc, (i, &b)| acc | ((b as u16) << i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fcs_on_known_data() {
        let mut crc = CRCu16::crc16ccitt_false();
        crc.digest(b"123456789");
        assert_eq!(crc.get_crc(), 0x29B1); // validación estándar
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
    fn fcs_to_bits() {
        let fcs_bits = FrameCheckingSequence(0b0000_1111_1111_0000).to_bits();
        let expected = vec![
            false, false, false, false, //
            true, true, true, true, //
            true, true, true, true, //
            false, false, false, false, //
        ];
        assert_eq!(fcs_bits, expected);
    }

    #[test]
    fn control_from_u8() {
        assert_eq!(
            Control::try_from(0b1100_1000).unwrap(),
            Control::Unnumbered {
                kind: UnnumberedType::Information,
                pf: true
            }
        );
        assert_eq!(
            Control::try_from(0b1100_0000).unwrap(),
            Control::Unnumbered {
                kind: UnnumberedType::Information,
                pf: false
            }
        );
        assert_eq!(
            Control::try_from(0b1100_1111).unwrap(),
            Control::Unnumbered {
                kind: UnnumberedType::Test,
                pf: true
            }
        );
        assert_eq!(
            Control::try_from(0b1100_0111).unwrap(),
            Control::Unnumbered {
                kind: UnnumberedType::Test,
                pf: false
            }
        );
    }

    #[test]
    fn control_from_u8_error() {
        assert!(matches!(
            Control::try_from(0b0000_0000),
            Err(DeframingError::InvalidControlFrameType)
        ));
        assert!(matches!(
            Control::try_from(0b1100_0001),
            Err(DeframingError::InvalidControlUnnumberedType)
        ));
    }

    #[test]
    fn control_to_u8() {
        assert_eq!(
            Byte::from(Control::Unnumbered {
                kind: UnnumberedType::Information,
                pf: true
            }),
            0b1100_1000,
        );
        assert_eq!(
            Byte::from(Control::Unnumbered {
                kind: UnnumberedType::Information,
                pf: false
            }),
            0b1100_0000,
        );
        assert_eq!(
            Byte::from(Control::Unnumbered {
                kind: UnnumberedType::Test,
                pf: true
            }),
            0b1100_1111,
        );
        assert_eq!(
            Byte::from(Control::Unnumbered {
                kind: UnnumberedType::Test,
                pf: false
            }),
            0b1100_0111,
        );
    }

    #[test]
    fn destuffed_frame_matches_original() {
        let original = vec![true, true, true, true, true, true, false, true];
        let stuffed = bit_stuff(&original);
        let destuffed = bit_destuff(&stuffed);
        assert_eq!(original, destuffed);
    }
}
