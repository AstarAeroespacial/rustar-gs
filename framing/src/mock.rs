use crate::deframe::Deframer;

#[derive(Debug)]
pub struct MockFrame {}

pub struct MockDeframingIterator<I, B> {
    input: I,
    _buffer: Vec<B>,
}

impl<I, B> Iterator for MockDeframingIterator<I, B>
where
    I: Iterator<Item = B>,
{
    type Item = MockFrame;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next().map(|_| MockFrame {})
    }
}

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

impl<I> Default for MockDeframer<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B, I> Deframer<B, MockFrame> for MockDeframer<I>
where
    I: Iterator<Item = B>,
{
    type Input = I;
    type Output = MockDeframingIterator<I, B>;

    fn frames(&self, input: Self::Input) -> Self::Output {
        MockDeframingIterator {
            input,
            _buffer: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_deframer_empty_input() {
        let deframer = MockDeframer::new();
        let input: Vec<u8> = vec![];
        let frames: Vec<MockFrame> = deframer.frames(input.into_iter()).collect();

        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_mock_deframer_single_item() {
        let deframer = MockDeframer::new();
        let input = vec![42u8];
        let frames: Vec<MockFrame> = deframer.frames(input.into_iter()).collect();

        assert_eq!(frames.len(), 1);
    }

    #[test]
    fn test_mock_deframer_basic() {
        let deframer = MockDeframer::new();
        let input = vec![1u8, 2, 3, 4, 5];
        let frames: Vec<MockFrame> = deframer.frames(input.into_iter()).collect();

        assert_eq!(frames.len(), 5);
    }
}
