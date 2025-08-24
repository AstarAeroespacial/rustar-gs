use crate::Demodulator;

pub struct ExampleDemod {}

impl ExampleDemod {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct ExampleDemodIterator<I>
where
    I: Iterator<Item = f64>,
{
    inner: I,
}

impl<I> Iterator for ExampleDemodIterator<I>
where
    I: Iterator<Item = f64>,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.next().is_some() {
            Some(true)
        } else {
            None
        }
    }
}

impl<I> Demodulator<I> for ExampleDemod
where
    I: Iterator<Item = f64>,
{
    type Output = ExampleDemodIterator<I>;

    fn bits(&self, input: I) -> Self::Output {
        ExampleDemodIterator { inner: input }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let demodulator = ExampleDemod {};
        let mut bits = demodulator.bits(vec![1f64, 2f64].into_iter());

        assert_eq!(bits.next(), Some(true));
        assert_eq!(bits.next(), Some(true));
        assert_eq!(bits.next(), None);
    }
}
