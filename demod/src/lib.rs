pub mod afsk1200;

pub trait Demodulator<I>
where
    I: Iterator<Item = Vec<f64>>,
{
    type Output: Iterator<Item = Vec<bool>>;

    fn bits(&self, input: I) -> Self::Output;
}
