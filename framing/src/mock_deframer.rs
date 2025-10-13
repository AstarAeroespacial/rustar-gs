use crate::{deframer::Deframer, frame::Frame};

pub struct MockDeframer<I> {
    payload: Option<Vec<u8>>,
    _phantom: std::marker::PhantomData<I>,
}

impl<I> MockDeframer<I> {
    pub fn new(payload: impl Into<Option<Vec<u8>>>) -> Self {
        let payload = payload.into();

        Self {
            payload: payload.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct MockDeframerIterator<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    payload: Option<Vec<u8>>,
    input: I,
}

impl<I> Iterator for MockDeframerIterator<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    type Item = Frame;

    // Returns a frame every 10 bit reads.
    fn next(&mut self) -> Option<Self::Item> {
        for _ in 0..10 {
            if self.input.next().is_none() {
                return None;
            }
        }

        println!("[DEFRAMER] Yielding mock frame");

        return Some(Frame::new(self.payload.clone()));
    }
}

impl<I> Deframer<Vec<bool>, Frame> for MockDeframer<I>
where
    I: Iterator<Item = Vec<bool>>,
{
    type Input = I;
    type Output = MockDeframerIterator<I>;

    fn frames(&self, input: Self::Input) -> Self::Output {
        MockDeframerIterator {
            input,
            payload: self.payload.clone(),
        }
    }
}
