use crate::deframe::Deframer;

/// An HDLC frame.
/// TODO: document
#[derive(Debug)]
pub struct HdlcFrame {}

pub struct HdlcDeframingIterator<I, B> {
    input: I,
    buffer: Vec<B>,
}

impl<I, B> Iterator for HdlcDeframingIterator<I, B>
where
    I: Iterator<Item = B>,
{
    type Item = HdlcFrame;

    fn next(&mut self) -> Option<Self::Item> {
        // Aquí va la lógica específica de HDLC deframing
        todo!()
    }
}

pub struct HdlcDeframer<I> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I> HdlcDeframer<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<B, I> Deframer<B, HdlcFrame> for HdlcDeframer<I>
where
    I: Iterator<Item = B>,
{
    type Input = I;
    type Output = HdlcDeframingIterator<I, B>;

    fn frames(&self, input: Self::Input) -> Self::Output {
        HdlcDeframingIterator {
            input,
            buffer: Vec::new(),
        }
    }
}
