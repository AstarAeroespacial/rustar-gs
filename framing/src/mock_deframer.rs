use crate::{deframer::Deframer, frame::Frame};

pub struct MockDeframer<I> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I> MockDeframer<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct MockDeframerIterator<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    input: I,
}

impl<I> Iterator for MockDeframerIterator<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    type Item = Frame;

    // Returns a frame every 100 bit reads.
    fn next(&mut self) -> Option<Self::Item> {
        for _ in 0..100 {
            if self.input.next().is_none() {
                return None;
            }
        }

        println!("[DEFRAMER] Yielding mock frame");

        return Some(Frame::new(None));
    }
}

impl<I> Deframer<Vec<bool>, Frame> for MockDeframer<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    type Input = I;
    type Output = MockDeframerIterator<I>;

    fn frames(&self, input: Self::Input) -> Self::Output {
        MockDeframerIterator { input }
    }
}
