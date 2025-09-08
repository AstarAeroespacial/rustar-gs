use crate::Packetizer;
use framing::frame::Frame;
use telemetry::TelemetryRecord;

/// Iterator that converts frames to packets.
struct TelemetryRecordIterator<I>
where
    I: Iterator<Item = Frame>,
{
    input: I,
}

impl<I> TelemetryRecordIterator<I>
where
    I: Iterator<Item = Frame>,
{
    /// Creates a new packetizer iterator.
    pub fn new(input: I) -> Self {
        Self { input }
    }
}

/// Iterator implementation that converts Frames to TelemetryPackets
impl<I> Iterator for TelemetryRecordIterator<I>
where
    I: Iterator<Item = Frame>,
{
    type Item = TelemetryRecord;

    fn next(&mut self) -> Option<Self::Item> {
        for frame in self.input.by_ref() {
            if let Some(info) = frame.info {
                if let Ok(packet) = serde_json::from_str(&String::from_utf8_lossy(&info)) {
                    return Some(packet);
                }
            }
        }
        None
    }
}

/// Telemetry packetizer implementation for HDLC Frame to TelemetryPacket conversion.
pub struct TelemetryRecordPacketizer;

impl Default for TelemetryRecordPacketizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryRecordPacketizer {
    pub fn new() -> Self {
        Self
    }
}

impl Packetizer<Frame, TelemetryRecord> for TelemetryRecordPacketizer {
    fn packets<I>(&self, input: I) -> impl Iterator<Item = TelemetryRecord>
    where
        I: Iterator<Item = Frame>,
    {
        TelemetryRecordIterator::new(input)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_telemetry_packetizer_empty() {
        let packetizer = TelemetryRecordPacketizer::new();
        let frames: Vec<Frame> = vec![];
        let packets: Vec<TelemetryRecord> = packetizer.packets(frames.into_iter()).collect();
        assert_eq!(packets.len(), 0);
    }

    #[test]
    fn test_packetizer_single_valid_frame() {
        let packetizer = TelemetryRecordPacketizer::new();

        // Create a frame with valid telemetry data as JSON
        let telemetry_record =
            TelemetryRecord::with_id("SAT001".to_string(), 1234567890, 25.5, 12.0, 150.0, 85);

        // Convert record to JSON bytes to simulate frame info
        let json_data = serde_json::to_string(&telemetry_record).unwrap();
        let packet_bytes = json_data.into_bytes();

        let frame = Frame::new(Some(packet_bytes));
        let frames = vec![frame];

        let packets: Vec<TelemetryRecord> = packetizer.packets(frames.into_iter()).collect();

        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].id, "SAT001");
        assert_eq!(packets[0].timestamp, 1234567890);
        assert_eq!(packets[0].temperature, 25.5);
        assert_eq!(packets[0].voltage, 12.0);
        assert_eq!(packets[0].current, 150.0);
        assert_eq!(packets[0].battery_level, 85);
    }

    #[test]
    fn test_packetizer_frame_without_info() {
        let packetizer = TelemetryRecordPacketizer::new();

        let frame = Frame::new(None);
        let frames = vec![frame];

        let packets: Vec<TelemetryRecord> = packetizer.packets(frames.into_iter()).collect();
        assert_eq!(packets.len(), 0); // No packets should be produced
    }

    #[test]
    fn test_packetizer_multiple_frames() {
        let packetizer = TelemetryRecordPacketizer::new();

        // Create valid telemetry records
        let telemetry_record1 =
            TelemetryRecord::with_id("SAT001".to_string(), 1234567890, 25.5, 12.0, 150.0, 85);

        let telemetry_record2 =
            TelemetryRecord::with_id("SAT002".to_string(), 1234567891, 26.0, 11.9, 145.0, 84);

        // Convert records to JSON bytes
        let json_data1 = serde_json::to_string(&telemetry_record1).unwrap();
        let packet1_bytes = json_data1.into_bytes();

        let json_data2 = serde_json::to_string(&telemetry_record2).unwrap();
        let packet2_bytes = json_data2.into_bytes();

        let frame1 = Frame::new(Some(packet1_bytes));
        let frame2 = Frame::new(Some(packet2_bytes));
        let frame3 = Frame::new(None); // Frame without info

        let frames = vec![frame1, frame2, frame3];

        let packets: Vec<TelemetryRecord> = packetizer.packets(frames.into_iter()).collect();
        assert_eq!(packets.len(), 2); // Only 2 packets should be produced (frame3 has no info)
        assert_eq!(packets[0].id, "SAT001");
        assert_eq!(packets[0].timestamp, 1234567890);
        assert_eq!(packets[1].id, "SAT002");
        assert_eq!(packets[1].timestamp, 1234567891);
    }

    #[test]
    fn test_packetizer_invalid_data() {
        let packetizer = TelemetryRecordPacketizer::new();

        // Create a frame with invalid JSON data
        let invalid_data = b"invalid json data".to_vec();

        let frame = Frame::new(Some(invalid_data));
        let frames = vec![frame];

        let packets: Vec<TelemetryRecord> = packetizer.packets(frames.into_iter()).collect();
        assert_eq!(packets.len(), 0); // No packets should be produced due to invalid data
    }
}
