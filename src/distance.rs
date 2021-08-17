use crate::map::Map;

pub trait Distance2D {
    fn evaluate(&self, pos1: (f64, f64), pos2: (f64, f64)) -> f64;
}

pub struct EuclideanDistance2D {}

impl Distance2D for EuclideanDistance2D {
    fn evaluate(&self, pos1: (f64, f64), pos2: (f64, f64)) -> f64 {
        ((pos2.0 - pos1.0).powi(2) + (pos2.1 - pos1.1).powi(2)).sqrt()
    }
}

pub struct ManhattanDistance2D {}

impl Distance2D for ManhattanDistance2D {
    fn evaluate(&self, pos1: (f64, f64), pos2: (f64, f64)) -> f64 {
        (pos2.0 - pos1.0).abs() + (pos2.1 - pos1.1).abs()
    }
}

pub struct NullDistance2D {}

impl Distance2D for NullDistance2D {
    fn evaluate(&self, _pos1: (f64, f64), _pos2: (f64, f64)) -> f64 {
        0.0
    }
}

pub struct EuclideanDistanceWHeight2D<'d> {
    pub height_map: &'d Map<f64>
}

impl<'d> Distance2D for EuclideanDistanceWHeight2D<'_> {
    fn evaluate(&self, pos1: (f64, f64), pos2: (f64, f64)) -> f64 {
        // Get associated height for the two position
        let ipos1 = (pos1.0 as i32, pos1.1 as i32);
        let ipos2 = (pos2.0 as i32, pos2.1 as i32);

        let height1 = self.height_map[ipos1] * 2.0;
        let height2 = self.height_map[ipos2] * 2.0;

        ((pos2.0 - pos1.0).powi(2) + (pos2.1 - pos1.1).powi(2) + (height2 - height1).powi(2)).sqrt()
    }
}