use crate::Demodulator;

pub struct AfskDemodulatorIterator<I, S> {
    input: I,
    buffer: Vec<S>,
}

impl<I, S> Iterator for AfskDemodulatorIterator<I, S>
where
    I: Iterator<Item = S>,
{
    type Item = Vec<bool>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct AfskDemodulator<I> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I> AfskDemodulator<I> {
    pub fn new() -> Self {
        todo!()
    }
}

impl<S, I> Demodulator<S, Vec<bool>> for AfskDemodulator<I>
where
    I: Iterator<Item = S>,
{
    type Input = I;
    type Output = AfskDemodulatorIterator<I, S>;

    fn bits(&self, input: Self::Input) -> Self::Output {
        AfskDemodulatorIterator {
            input,
            buffer: Vec::new(),
        }
    }
}
