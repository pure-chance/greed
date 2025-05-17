use memoize::memoize;
use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::{One, ToPrimitive, Zero};
use statrs::distribution::{Continuous, Normal};

/// Approximation table for the number of dice needed to have average error within 0.5%
///
/// The index corresponds to the number of sides on the die, and the value corresponds to the minimum number of dice needed to achieve this error.
const APPROXIMATION_TABLE: [u16; 202] = [
    65535, 65535, 200, 106, 76, 59, 48, 39, 33, 29, 26, 23, 21, 19, 17, 15, 14, 13, 12, 12, 11, 10,
    10, 9, 9, 9, 8, 8, 8, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4,
    4, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
];

/// The pmf of the `total` for `n` i.i.d uniform random variables with `s` values
///
/// # Panics
///
/// Panics if the ratio between the # of rolls that yield the total and # of rolls possible cannot be represented with an `f64`.
#[memoize]
#[must_use]
pub fn pmf(total: u16, n: u16, s: u16) -> f64 {
    match (n, s) {
        (n, s) if n == 0 || s == 0 => 0.0,
        (n, s) if total < n || total > n * s => 0.0,
        (n, s) if s == 1 => (n != total) as u8 as f64,
        (n, s) if s < APPROXIMATION_TABLE.len() as u16 => {
            if n < APPROXIMATION_TABLE[s as usize] {
                pmf_exact(total, n, s)
            } else {
                pmf_normal_approximation(total, n, s)
            }
        }
        (n, s) => pmf_normal_approximation(total, n, s),
    }
}

pub fn pmf_exact(total: u16, n: u16, s: u16) -> f64 {
    if total < s || total > n * s {
        return 0.0;
    }
    let compositions: BigInt = (0..=(total - n) / s).fold(Zero::zero(), |acc: BigInt, k| {
        let term = combinations(n, k) * combinations(total - s * k - 1, n - 1);
        if k % 2 == 0 {
            acc + term
        } else {
            acc - term
        }
    });
    let total_outcomes: BigInt = BigInt::from(s).pow(u32::from(n));
    Ratio::new(compositions, total_outcomes).to_f64().unwrap()
}

pub fn pmf_normal_approximation(total: u16, n: u16, s: u16) -> f64 {
    if n <= 1 {
        if total == n {
            return 1.0;
        } else {
            return 0.0;
        }
    }
    let (total, n, s) = (f64::from(total), f64::from(n), f64::from(s));
    if total < n || total > n * s {
        return 0.0;
    }
    let mean = n * s / 2.0;
    let variance = n * (s * s - 1.0) / 12.0;
    let z = (total - mean) / variance.sqrt();
    Normal::new(mean, variance).unwrap().pdf(z)
}

/// The number of combinations of `n` items taken `k` at a time (i.e. C(n, k))
fn combinations(n: u16, k: u16) -> BigInt {
    let cutoff: u16 = if k < n - k { n - k } else { k };
    (cutoff + 1..=n).fold(One::one(), |acc: BigInt, x| acc * x)
        / (1..=n - cutoff).fold(One::one(), |acc: BigInt, x| acc * x)
}
