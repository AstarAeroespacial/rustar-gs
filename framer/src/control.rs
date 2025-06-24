/// ## Supervisory Frame kinds
/// These frames are used to control the flow of information and manage the state of the connection. <br/>
/// They are identified by the last two bits of the control byte. <br/>
/// The first two bits indicate the frame type, and the last two bits indicate the supervisory kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisoryKind {
    RR,   // 00 Receive Ready
    REJ,  // 01 Reject
    RNR,  // 10 Receive Not Ready
    SREJ, // 11 Selective Reject
    Unknown(u8),
}

/// ## Unnumbered Frame types <br/>
/// These frames do not carry sequence numbers and are used for control purposes. <br/>
/// They are identified by the last four bits of the control byte. <br/>
/// The first two bits indicate the frame type, and the last two bits are reserved for future use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UFrameType {
    SABM, // Set Asynchronous Balanced Mode
    UA,   // Unnumbered Acknowledge
    DISC, // Disconnect
    DM,   // Disconnected Mode
    UI,   // Unnumbered Information
    FRMR, // Frame Reject
    TEST, // Test frame
    XID,  // Exchange Identification
    Unknown(u8),
}

/// Control Field for HDLC frames <br/>
/// This field is used to control the flow of data in HDLC frames. <br/>
/// It can represent different types of frames: I-frames, S-frames, and U-frames. <br/>
/// The type of frame is determined by the last two bits of the control byte. <br/>
/// The first two bits indicate the frame type, and the remaining bits carry sequence numbers or control information as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlField {
    // IFrame: Information Frame
    IFrame {
        ns: u8,   // Send Sequence Number
        nr: u8,   // Receive Sequence Number
        pf: bool, // Poll/Final bit
    },
    // SFrame: Supervisory Frame
    SFrame {
        kind: SupervisoryKind,
        nr: u8,
        pf: bool,
    },
    // UFrame: Unnumbered Frame
    UFrame {
        code: UFrameType,
        pf: bool,
    },
    Unknown(u8),
}

impl ControlField {
    /// Creates a new ControlField from a byte.
    /// The byte is expected to be in the format defined by the HDLC protocol.
    pub fn from_u8(byte: u8) -> Self {
        // Verificar primero si es I-frame (último bit = 0)
        if (byte & 0b1) == 0 {
            // I-frame: último bit 0
            let ns = (byte >> 1) & 0b111;
            let pf = (byte >> 4) & 1 != 0;
            let nr = (byte >> 5) & 0b111;
            ControlField::IFrame { ns, nr, pf }
        } else {
            // Si último bit = 1, verificar los últimos 2 bits
            match byte & 0b11 {
                0b01 => {
                    // S-frame: últimos dos bits 01
                    let kind = match (byte >> 2) & 0b11 {
                        0b00 => SupervisoryKind::RR,
                        0b01 => SupervisoryKind::REJ,
                        0b10 => SupervisoryKind::RNR,
                        0b11 => SupervisoryKind::SREJ,
                        b => SupervisoryKind::Unknown(b),
                    };
                    let pf = (byte >> 4) & 1 != 0;
                    let nr = (byte >> 5) & 0b111;
                    ControlField::SFrame { kind, nr, pf }
                }
                0b11 => {
                    // U-frame: últimos dos bits 11
                    let pf = (byte >> 4) & 1 != 0;
                    // Enmascarar el bit P/F para identificar el tipo
                    let masked = byte & 0b1110_1111;
                    let code = match masked {
                        0b0010_1111 => UFrameType::SABM,
                        0b0110_0011 => UFrameType::UA,
                        0b0100_0011 => UFrameType::DISC,
                        0b0000_1111 => UFrameType::DM,
                        0b0000_0011 => UFrameType::UI,
                        0b1000_0111 => UFrameType::FRMR,
                        0b0000_0111 => UFrameType::TEST,
                        0b1010_1111 => UFrameType::XID,
                        _ => UFrameType::Unknown(masked),
                    };
                    ControlField::UFrame { code, pf }
                }
                _ => ControlField::Unknown(byte),
            }
        }
    }

    /// Converts the ControlField to a byte representation.
    /// The byte is formatted according to the HDLC protocol specifications.
    pub fn to_u8(&self) -> u8 {
        match self {
            ControlField::IFrame { ns, nr, pf } => {
                let mut byte = 0u8;
                byte |= (ns & 0b111) << 1;
                if *pf {
                    byte |= 1 << 4;
                }
                byte |= (nr & 0b111) << 5;
                byte
            }
            ControlField::SFrame { kind, nr, pf } => {
                let mut byte = 0b01;
                let kind_bits = match kind {
                    SupervisoryKind::RR => 0b00,
                    SupervisoryKind::REJ => 0b01,
                    SupervisoryKind::RNR => 0b10,
                    SupervisoryKind::SREJ => 0b11,
                    SupervisoryKind::Unknown(b) => *b,
                };
                byte |= kind_bits << 2;
                if *pf {
                    byte |= 1 << 4;
                }
                byte |= (nr & 0b111) << 5;
                byte
            }
            ControlField::UFrame { code, pf } => {
                let mut byte = match code {
                    UFrameType::SABM => 0b0010_1111,
                    UFrameType::UA => 0b0110_0011,
                    UFrameType::DISC => 0b0100_0011,
                    UFrameType::DM => 0b0000_1111,
                    UFrameType::UI => 0b0000_0011,
                    UFrameType::FRMR => 0b1000_0111,
                    UFrameType::TEST => 0b0000_0111,
                    UFrameType::XID => 0b1010_1111,
                    UFrameType::Unknown(b) => *b,
                };
                if *pf {
                    byte |= 1 << 4;
                }
                byte
            }
            ControlField::Unknown(b) => *b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iframe_from_u8() {
        // I-frame: ns=3, nr=5, pf=true
        // Formato: nr(3bits) | pf(1bit) | ns(3bits) | 0
        // 101 | 1 | 011 | 0 = 0b10110110 = 0xB6
        let byte = 0b10110110;
        let control = ControlField::from_u8(byte);

        match control {
            ControlField::IFrame { ns, nr, pf } => {
                assert_eq!(ns, 3);
                assert_eq!(nr, 5);
                assert_eq!(pf, true);
            }
            _ => panic!("Expected IFrame"),
        }
    }

    #[test]
    fn test_iframe_to_u8() {
        let control = ControlField::IFrame {
            ns: 3,
            nr: 5,
            pf: true,
        };
        let byte = control.to_u8();
        assert_eq!(byte, 0b10110110);
    }

    #[test]
    fn test_sframe_rr() {
        // S-frame RR: nr=4, pf=false
        // Formato: nr(3bits) | pf(1bit) | kind(2bits) | 01
        // 100 | 0 | 00 | 01 = 0b10000001 = 0x81
        let byte = 0b10000001;
        let control = ControlField::from_u8(byte);

        match control {
            ControlField::SFrame { kind, nr, pf } => {
                assert_eq!(kind, SupervisoryKind::RR);
                assert_eq!(nr, 4);
                assert_eq!(pf, false);
            }
            _ => panic!("Expected SFrame"),
        }
    }

    #[test]
    fn test_uframe_ui() {
        // U-frame UI: pf=false
        // UI = 0b0000_0011
        let byte = 0b00000011;
        let control = ControlField::from_u8(byte);

        match control {
            ControlField::UFrame { code, pf } => {
                assert_eq!(code, UFrameType::UI);
                assert_eq!(pf, false);
            }
            _ => panic!("Expected UFrame"),
        }
    }

    #[test]
    fn test_unknown_frame_types() {
        let unknown_uframe = 0b11010011; // U-frame con patrón desconocido
        let control = ControlField::from_u8(unknown_uframe);

        match control {
            ControlField::UFrame { code, pf: _ } => {
                match code {
                    UFrameType::Unknown(_) => (), // Esto es lo que esperamos
                    _ => panic!("Expected Unknown UFrameType, got {:?}", code),
                }
            }
            _ => panic!("Expected UFrame with Unknown type"),
        }
    }
}
