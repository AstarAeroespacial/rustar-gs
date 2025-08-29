/// A trait for converting a stream of bits into frames.
///
/// This trait provides a lazy iterator-based approach to deframing, where
/// the actual deframing logic is implemented in the returned iterator's
/// `next()` method.
///
/// # Type Parameters
///
/// * `B` - The type of bits (e.g., `bool`, `Vec<bool>`, custom `Bit` type)
/// * `F` - The type of frames to be produced (e.g., `HdlcFrame`, `Ax25Frame`)
pub trait Deframer<B, F> {
    /// The input iterator type that yields bits.
    type Input: Iterator<Item = B>;
    /// The output iterator type that yields complete frames.
    type Output: Iterator<Item = F>;

    /// Creates an iterator that converts bits into frames.
    ///
    /// This method does not perform any deframing immediately. Instead, it
    /// returns an iterator that will perform the deframing lazily as frames
    /// are requested.
    ///
    /// # Arguments
    ///
    /// * `input` - An iterator of bits to be deframed
    ///
    /// # Returns
    ///
    /// An iterator that yields frames as they are successfully deframed
    /// from the input bit stream.
    fn frames(&self, input: Self::Input) -> Self::Output;
}
