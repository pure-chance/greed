use memoize::memoize;
use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::{One, ToPrimitive, Zero};

/// The pmf of the `total` for `n` i.i.d uniform random variables with `s` values
///
/// # Panics
///
/// Panics if the ratio between the # of rolls that yield the total and # of rolls possible cannot be represented with an `f64`.
#[memoize]
#[must_use]
pub fn pmf(total: u16, n: u16, s: u16) -> f64 {
    if total == 0 {
        return 0.0;
    }
    if total > n * s {
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

/// The number of combinations of `n` items taken `k` at a time (i.e. C(n, k))
fn combinations(n: u16, k: u16) -> BigInt {
    let cutoff: u16 = if k < n - k { n - k } else { k };
    (cutoff + 1..=n).fold(One::one(), |acc: BigInt, x| acc * x)
        / (1..=n - cutoff).fold(One::one(), |acc: BigInt, x| acc * x)
}
