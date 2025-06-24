/*
 * Flag | Address | Control | Payload | FCS | Flag
 * 0x7E |   1B    |   1B    |  nB     |  2B | 0x7E
 */

pub struct HdlcFrame {
    address: u8,
    control: u8,
    payload: Vec<u8>,
    fcs: u16,
}
