use crate::frame::Frame;
use crate::packets::telemetry::TelemetryPacket;

/// An iterator that takes frames and yields telemetry packets.
/// 
/// This implements a simple telemetry encoding with one packet per frame,
/// as specified in the requirements.
pub(crate) struct Packetizer<I> 
where
    I: Iterator<Item = Frame>,
{
    frames: I,
}

impl<I> Packetizer<I> 
where
    I: Iterator<Item = Frame>,
{
    /// Creates a new packetizer from an iterator of frames.
    pub(crate) fn new(frames: I) -> Self {
        Self { frames }
    }
}

impl<I> Iterator for Packetizer<I> 
where
    I: Iterator<Item = Frame>,
{
    type Item = TelemetryPacket;

    fn next(&mut self) -> Option<Self::Item> {
        // Process frames one by one, yielding telemetry packets
        while let Some(frame) = self.frames.next() {
            if let Some(info) = frame.info {
                if let Ok(packet) = TelemetryPacket::try_from(info) {
                    return Some(packet);
                }
                // If conversion fails, continue to the next frame
            }
            // If frame has no info, continue to the next frame
        }
        None
    }
}

/// Creates a packetizer iterator from an iterator of frames.
/// 
/// This is a convenience function that creates a new Packetizer instance.
pub(crate) fn packetize<I>(frames: I) -> Packetizer<I> 
where
    I: Iterator<Item = Frame>,
{
    Packetizer::new(frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Control, Frame, UnnumberedType};
    use crate::packets::telemetry::{TelemetryPacket, TelemetryData};

    #[test]
    fn test_packetizer_empty() {
        let frames: Vec<Frame> = vec![];
        let packets: Vec<TelemetryPacket> = packetize(frames.into_iter()).collect();
        assert_eq!(packets.len(), 0);
    }

    #[test]
    fn test_packetizer_single_valid_frame() {
        // Create a frame with valid telemetry data
        let telemetry_data = TelemetryData::new(1234567890, 25.5, 12000.0, 150.0, 85);
        let telemetry_packet = TelemetryPacket {
            pkt_type: 0x01,
            length: 17,
            payload: telemetry_data,
        };
        
        // Convert packet to bytes to simulate frame info
        let mut packet_bytes = vec![telemetry_packet.pkt_type];
        packet_bytes.extend_from_slice(&telemetry_packet.length.to_be_bytes());
        packet_bytes.extend_from_slice(&telemetry_packet.payload.to_bytes());

        let control = Control::Unnumbered { 
            kind: UnnumberedType::Information, 
            pf: false 
        };
        let frame = Frame::new(0x01, control, Some(packet_bytes));
        let frames = vec![frame];

        let packets: Vec<TelemetryPacket> = packetize(frames.into_iter()).collect();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].pkt_type, 0x01);
        assert_eq!(packets[0].length, 17);
    }

    #[test]
    fn test_packetizer_frame_without_info() {
        let control = Control::Unnumbered { 
            kind: UnnumberedType::Information, 
            pf: false 
        };
        let frame = Frame::new(0x01, control, None);
        let frames = vec![frame];

        let packets: Vec<TelemetryPacket> = packetize(frames.into_iter()).collect();
        assert_eq!(packets.len(), 0); // No packets should be produced
    }

    #[test]
    fn test_packetizer_multiple_frames() {
        // Create valid telemetry data
        let telemetry_data1 = TelemetryData::new(1234567890, 25.5, 12000.0, 150.0, 85);
        let telemetry_packet1 = TelemetryPacket {
            pkt_type: 0x01,
            length: 17,
            payload: telemetry_data1,
        };
        
        let telemetry_data2 = TelemetryData::new(1234567891, 26.0, 11900.0, 145.0, 84);
        let telemetry_packet2 = TelemetryPacket {
            pkt_type: 0x01,
            length: 17,
            payload: telemetry_data2,
        };

        // Convert packets to bytes
        let mut packet1_bytes = vec![telemetry_packet1.pkt_type];
        packet1_bytes.extend_from_slice(&telemetry_packet1.length.to_be_bytes());
        packet1_bytes.extend_from_slice(&telemetry_packet1.payload.to_bytes());

        let mut packet2_bytes = vec![telemetry_packet2.pkt_type];
        packet2_bytes.extend_from_slice(&telemetry_packet2.length.to_be_bytes());
        packet2_bytes.extend_from_slice(&telemetry_packet2.payload.to_bytes());

        let control = Control::Unnumbered { 
            kind: UnnumberedType::Information, 
            pf: false 
        };
        
        let frame1 = Frame::new(0x01, control.clone(), Some(packet1_bytes));
        let frame2 = Frame::new(0x01, control.clone(), Some(packet2_bytes));
        let frame3 = Frame::new(0x01, control, None); // Frame without info
        
        let frames = vec![frame1, frame2, frame3];

        let packets: Vec<TelemetryPacket> = packetize(frames.into_iter()).collect();
        assert_eq!(packets.len(), 2); // Only 2 packets should be produced (frame3 has no info)
        assert_eq!(packets[0].payload.timestamp, 1234567890);
        assert_eq!(packets[1].payload.timestamp, 1234567891);
    }

    #[test]
    fn test_packetizer_invalid_data() {
        // Create a frame with invalid telemetry data (too short)
        let invalid_data = vec![0x01]; // Only type, no length or payload
        let control = Control::Unnumbered { 
            kind: UnnumberedType::Information, 
            pf: false 
        };
        let frame = Frame::new(0x01, control, Some(invalid_data));
        let frames = vec![frame];

        let packets: Vec<TelemetryPacket> = packetize(frames.into_iter()).collect();
        assert_eq!(packets.len(), 0); // No packets should be produced due to invalid data
    }
}