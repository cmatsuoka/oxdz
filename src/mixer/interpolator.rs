
const SHIFT: i32 = 16;

pub trait InterpolatorBase {
    fn name() -> &'static str;
}

pub trait Interpolate<T> {
    fn get_sample(&self, &[T], i32) -> i32;
}

pub enum AnyInterpolator {
    NearestNeighbor(NearestNeighbor),
    Linear(Linear),
}

// Nearest neighbor interpolator
pub struct NearestNeighbor;

impl InterpolatorBase for NearestNeighbor {
    fn name() -> &'static str {
        "nearest neighbor"
    }
}

impl Interpolate<i8> for NearestNeighbor {
    fn get_sample(&self, i: &[i8], _frac: i32) -> i32 {
        (i[1] as i32) << 8
    }
}

impl Interpolate<i16> for NearestNeighbor {
    fn get_sample(&self, i: &[i16], _frac: i32) -> i32 {
        i[1] as i32
    }
}


// Linear interpolator
pub struct Linear;

impl InterpolatorBase for Linear {
    fn name() -> &'static str {
        "linear"
    }
}

impl Interpolate<i8> for Linear {
    fn get_sample(&self, i: &[i8], frac: i32) -> i32 {
        let l1 = (i[1] as i32) << 8;
        let dt = (i[2] as i32) << 8 - l1;
        l1 as i32 + (((frac >> 1) * dt as i32) >> (SHIFT - 1)) as i32
    }
}

impl Interpolate<i16> for Linear {
    fn get_sample(&self, i: &[i16], frac: i32) -> i32 {
        let l1 = i[1] as i32;
        let dt = i[2] as i32 - l1;
        l1 as i32 + (((frac >> 1) * dt as i32) >> (SHIFT - 1)) as i32
    }
}

