mod demodulator;
mod modulator;

pub use demodulator::Demodulator;
pub use modulator::Modulator;

trait BitSink {}
trait SampleSource {}
