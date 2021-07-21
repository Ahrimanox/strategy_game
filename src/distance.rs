pub trait Distance2D {
    fn evaluate(&self, pos1: (f64, f64), pos2: (f64, f64)) -> f64;
}

pub struct EuclideanDistance2D {}

impl Distance2D for EuclideanDistance2D {
    fn evaluate(&self, pos1: (f64, f64), pos2: (f64, f64)) -> f64 {
        ((pos2.0 - pos1.0).powi(2) + (pos2.1 - pos1.1).powi(2)).sqrt()
    }
}