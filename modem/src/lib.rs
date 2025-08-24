pub mod afsk1200;
pub mod demodulator;
pub mod modulator;

/// `S`: type of the samples input.
///
/// `B`: type of the returned bits.
pub trait Demodulator<S, B> {
    type Input: Iterator<Item = S>;
    type Output: Iterator<Item = B>;

    fn bits(&self, input: Self::Input) -> Self::Output;
}
