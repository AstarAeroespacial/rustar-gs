pub mod afsk1200;
pub mod example;
pub mod gr_mock;

pub trait Demodulator<I>
where
    I: Iterator<Item = Vec<f64>>,
{
    type Output: Iterator<Item = Vec<bool>>;

    fn bits(&self, input: I) -> Self::Output;
}
