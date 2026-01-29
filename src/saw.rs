pub trait Oscillator {
    fn next(&self) -> f64;
}

pub struct SawWave {
    freq: f64,
    clock: f64,
    phase: f64,
    reversed: bool,
}

impl Oscillator for SawWave {
    fn next(&self) -> f64 {
        ((self.clock + self.phase) * self.freq) % 1.0
    }
}
