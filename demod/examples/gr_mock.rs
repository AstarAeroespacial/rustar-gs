use demod::gr_mock::GrBitSource;

fn main() {
    let demod = GrBitSource::new();

    for bits in demod.into_iter() {
        println!("{:?}", bits);
    }
}
