pub trait Semiring {
    fn one() -> f64;
    fn null() -> f64;
    fn add(a: f64, b: f64) -> f64;
    fn multiply(a: f64, b: f64) -> f64;
}

pub struct ViterbiSemiring;

impl Semiring for ViterbiSemiring {
    fn one() -> f64 {
        1.0
    }

    fn null() -> f64 {
        0.0
    }

    fn add(a: f64, b: f64) -> f64 {
        a.max(b)
    }

    fn multiply(a: f64, b: f64) -> f64 {
        a * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viterbi() {
        assert_eq!(ViterbiSemiring::one(), 1.0);
        assert_eq!(ViterbiSemiring::null(), 0.0);
        assert_eq!(ViterbiSemiring::add(0.3, 0.7), 0.7);
        assert_eq!(ViterbiSemiring::multiply(0.5, 0.6), 0.3);
    }
}
