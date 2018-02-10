use mixer::SMIX_SHIFT;

pub trait Interpolate {
    fn name() -> &'static str;
    fn bsize() -> usize;
    fn get_sample(&self, &[i32], i32) -> i32;
}

pub enum Interpolator {
    Nearest,
    Linear,
}

// Nearest neighbor interpolator
pub struct Nearest;

impl Interpolate for Nearest {
    fn name() -> &'static str {
        "nearest neighbor"
    }

    fn bsize() -> usize {
        2
    }

    fn get_sample(&self, i: &[i32], _frac: i32) -> i32 {
        i[1]
    }
}


// Linear interpolator
pub struct Linear;

impl Interpolate for Linear {
    fn name() -> &'static str {
        "linear"
    }

    fn bsize() -> usize {
        2
    }

    fn get_sample(&self, i: &[i32], frac: i32) -> i32 {
        let l1 = i[0];
        let dt = i[1] - l1;
        l1 + (((frac >> 1) * dt) >> (SMIX_SHIFT - 1))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_nearest_i8() {
        let interp = Nearest;
        let i: &[i8] = &[0, 0x10, 0x40, 0x70];
        assert_eq!(interp.get_sample(i, 0), 0x1000);
        assert_eq!(interp.get_sample(i, 32767), 0x1000);
        assert_eq!(interp.get_sample(i, 65535), 0x1000);
    }

    #[test]
    fn test_interpolate_nearest_i16() {
        let interp = Nearest;
        let i: &[i16] = &[0, 0x1000, 0x4000, 0x7000];
        assert_eq!(interp.get_sample(i, 0), 0x1000);
        assert_eq!(interp.get_sample(i, 32767), 0x1000);
        assert_eq!(interp.get_sample(i, 65535), 0x1000);
    }

    #[test]
    fn test_interpolate_linear_i8() {
        let interp = Linear;
        let i: &[i8] = &[0, 0x10, 0x40, 0x70];
        assert_eq!(interp.get_sample(i, 0), 0x1000);
        assert_eq!(interp.get_sample(i, 32767), 0x27ff);
        assert_eq!(interp.get_sample(i, 65535), 0x3fff);
    }

    #[test]
    fn test_interpolate_linear_i16() {
        let interp = Linear;
        let i: &[i16] = &[0, 0x1000, 0x4000, 0x7000];
        assert_eq!(interp.get_sample(i, 0), 0x1000);
        assert_eq!(interp.get_sample(i, 32767), 0x27ff);
        assert_eq!(interp.get_sample(i, 65535), 0x3fff);
    }
}
