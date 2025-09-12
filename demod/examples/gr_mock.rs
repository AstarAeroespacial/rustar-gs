use demod::gr_mock::GrMockDemodIterator;

fn main() {
    let demod = GrMockDemodIterator::new();

    for bits in demod.into_iter() {
        println!("{:?}", bits);
    }
}
