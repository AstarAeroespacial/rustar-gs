pub mod packetizer;

pub trait Packetizer<F, P> {
    /// Creates an iterator that converts frames into packets.
    fn packets<I>(&self, input: I) -> impl Iterator<Item = P>
    where
        I: Iterator<Item = F>;
}
