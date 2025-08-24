use crate::Demodulator;

pub struct ExampleDemod {}

struct ExampleDemodIterator<I>
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
        todo!()
    }
}

impl<I> Demodulator<I> for ExampleDemod
where
    I: Iterator<Item = f64>,
{
    // type Output = BitIteratorOne;
    type Output = ExampleDemodIterator<I>;

    fn bits(&self, input: I) -> Self::Output {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let demodulator = ExampleDemod {};
        let bits = demodulator.bits([0f64; 10]);

        for bit in demodulator {
            dbg!(&bit);
        }
    }
}
