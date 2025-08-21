mod example;

pub trait Demodulator<I>
where
    I: Iterator<Item = f64>,
{
    type Output: Iterator<Item = bool>;

    fn bits(&self, input: I) -> Self::Output;
}
