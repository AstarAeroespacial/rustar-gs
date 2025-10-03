use crate::Demodulator;

pub struct ExampleDemod {}

impl Default for ExampleDemod {
    fn default() -> Self {
        Self::new()
    }
}

impl ExampleDemod {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct ExampleDemodIterator<I>
where
    I: Iterator<Item = Vec<f64>>,
{
    inner: I,
}

impl<I> Iterator for ExampleDemodIterator<I>
where
    I: Iterator<Item = Vec<f64>>,
{
    type Item = Vec<bool>;

    // Returns a bit every 100 sample reads.
    fn next(&mut self) -> Option<Self::Item> {
        for _ in 0..100 {
            if self.inner.next().is_none() {
                return None;
            }
        }

        return Some(vec![true]);
    }
}

impl<I> Demodulator<I> for ExampleDemod
where
    I: Iterator<Item = Vec<f64>>,
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
        let mut bits = demodulator.bits(vec![vec![1f64], vec![2f64]].into_iter());

        assert_eq!(bits.next(), Some(vec![true]));
        assert_eq!(bits.next(), Some(vec![true]));
        assert_eq!(bits.next(), None);
    }
}
