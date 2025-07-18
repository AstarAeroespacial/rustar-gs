use crc_any::CRCu16;

type Bit = bool;
type Byte = u8;

/// Unnumbered format commands/responses.
#[derive(Debug, Clone, PartialEq)]
enum UnnumberedType {
    Information, // UI - Unnumbered Information
    Test,        // TEST - Unnumbered Test
}

/// Defines the function of the frame.
#[derive(Debug, Clone, PartialEq)]
enum Control {
    /// The U format is used to provide additional data link control
    /// functions and unnumbered information transfer.
    Unnumbered { kind: UnnumberedType, pf: Bit },
}

#[derive(Debug)]
enum DeframingError {
    InvalidControlFrameType,
    InvalidControlUnnumberedType,
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

/// Represents an HDLC frame.
///
/// ## Fields
/// - `address`: The address field of the frame.
/// - `control`: The control field, representing frame's function.
/// - `info`: Optional payload data contained in the frame.
/// - `fcs`: Frame Check Sequence for error detection.
struct Frame {
    address: Byte,
    control: Control,
    info: Option<Vec<Byte>>,
    fcs: FrameCheckingSequence,
}

struct FrameCheckingSequence(u16);

impl FrameCheckingSequence {
    /// Converts a FrameCheckingSequence into a vector of bits.
    pub fn to_bits(&self) -> Vec<Bit> {
        (0..16).map(|i| (self.0 & (1 << i)) != 0).collect()
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

    /// Helper to convert a Byte to a Vec<Bit>, Least Significant Bit first
    fn byte_to_bits(byte: Byte) -> Vec<Bit> {
        (0..8).map(|i| (byte & (1 << i)) != 0).collect()
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

    /// Converts a Frame into a vector of bits.
    pub fn to_bits(&self) -> Vec<Bit> {
        let mut raw_bits = Vec::new();

        // Add fields without bit stuffing
        raw_bits.extend(Self::byte_to_bits(self.address));
        raw_bits.extend(Self::byte_to_bits(self.control.clone().into()));
        if let Some(info) = &self.info {
            for byte in info {
                raw_bits.extend(Self::byte_to_bits(*byte));
            }
        }
        raw_bits.extend(self.fcs.to_bits());

        // Apply bit stuffing to the entire content between flags
        let stuffed_bits = Self::bit_stuff(&raw_bits);

        // Build frame with flags
        let mut bits = Vec::new();
        bits.extend(Self::byte_to_bits(FLAG));
        bits.extend(stuffed_bits);
        bits.extend(Self::byte_to_bits(FLAG));

        bits
    }
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
        assert_eq!(Frame::bit_stuff(&input), expected);
    }

    #[test]
    fn stuffing_exactly_five_ones() {
        // Cinco unos consecutivos → se inserta un 0
        let input = vec![true, true, true, true, true];
        let expected = vec![true, true, true, true, true, false];
        assert_eq!(Frame::bit_stuff(&input), expected);
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
        assert_eq!(Frame::bit_stuff(&input), expected);
    }

    #[test]
    fn stuffing_multiple_groups() {
        // Dos grupos de cinco unos
        let input = vec![true, true, true, true, true, true, true, true, true, true];
        let expected = vec![
            true, true, true, true, true, false, true, true, true, true, true, false,
        ];
        assert_eq!(Frame::bit_stuff(&input), expected);
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
        assert_eq!(Frame::bit_stuff(&input), expected);
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
}
