use crate::control::ControlField;

/*
 * Flag | Address | Control | Payload | FCS | Flag
 * 0x7E |   1B    |   1B    |  nB     |  2B | 0x7E
 */

pub struct HdlcFrame {
    address: u8,
    control: ControlField,
    payload: Vec<u8>,
    fcs: u16,
}

impl HdlcFrame {
    pub fn new(address: u8, control: ControlField, payload: Vec<u8>, fcs: u16) -> Self {
        HdlcFrame {
            address,
            control,
            payload,
            fcs,
        }
    }

    pub fn to_bits(&self) -> Vec<bool> {
        let mut bits = Vec::new();

        // Flag inicial (0x7E = 01111110)
        bits.extend_from_slice(&Self::byte_to_bits(0x7E));

        // Address
        bits.extend_from_slice(&Self::byte_to_bits(self.address));

        // Control
        bits.extend_from_slice(&Self::byte_to_bits(self.control.to_u8()));

        // Payload
        for byte in &self.payload {
            bits.extend_from_slice(&Self::byte_to_bits(*byte));
        }

        // FCS (2 bytes, big-endian)
        let fcs_bytes = self.fcs.to_be_bytes();
        for byte in &fcs_bytes {
            bits.extend_from_slice(&Self::byte_to_bits(*byte));
        }

        // Flag final (0x7E = 01111110)
        bits.extend_from_slice(&Self::byte_to_bits(0x7E));

        bits
    }

    // Función auxiliar para convertir un byte a array de 8 bits (MSB first)
    fn byte_to_bits(byte: u8) -> [bool; 8] {
        [
            (byte & 0b10000000) != 0,
            (byte & 0b01000000) != 0,
            (byte & 0b00100000) != 0,
            (byte & 0b00010000) != 0,
            (byte & 0b00001000) != 0,
            (byte & 0b00000100) != 0,
            (byte & 0b00000010) != 0,
            (byte & 0b00000001) != 0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{ControlField, UFrameType};

    #[test]
    fn test_frame_to_bits() {
        let frame = HdlcFrame::new(
            0x01,
            ControlField::UFrame {
                code: UFrameType::UI,
                pf: false,
            },
            vec![0xAA, 0xBB],
            0x1234,
        );

        let bits = frame.to_bits();

        // Verificar que comience y termine con flags
        assert_eq!(
            &bits[0..8],
            &[false, true, true, true, true, true, true, false]
        ); // Flag inicial
        assert_eq!(
            &bits[bits.len() - 8..],
            &[false, true, true, true, true, true, true, false]
        ); // Flag final

        // Verificar address
        assert_eq!(&bits[8..16], &HdlcFrame::byte_to_bits(0x01));

        // El frame debería tener: Flag(8) + Address(8) + Control(8) + Payload(16) + FCS(16) + Flag(8) = 64 bits
        assert_eq!(bits.len(), 64);
    }
}
