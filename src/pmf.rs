/// Lookup table for dice-roll probability mass functions (PMFs).
///
/// Precomputes and stores the PMF for every dice count from 0 to an upper
/// bound, enabling O(1) lookup of P(sum = k | n dice, s sides). This is the
/// performance-critical data structure of the optimizer — PMF lookups happen
/// millions of times during policy computation.
///
/// # Layout
///
/// All PMF values are stored in a single contiguous `Box<[f64]>`. A separate
/// `offsets` array records where each dice-count's PMF begins.
///
/// For `n` dice with `s` sides the possible sums run from `n` to `n * s`,
/// giving `n * (s - 1) + 1` values. Indexing with `(n, total)` is translated to
/// `data[offsets[n] + (total - n)]`.
#[derive(Default, Debug, Clone)]
pub struct PMFLookup {
    /// Flat array containing all PMF data.
    data: Box<[f64]>,
    /// Starting offsets for each n-dice PMF.
    offsets: Box<[usize]>,
    /// Maximum number of dice.
    max_n: u32,
}

impl PMFLookup {
    /// Precompute all PMFs required by a game with the given parameters.
    ///
    /// Generates PMFs for 0 through `max_n` dice, where `max_n` is the
    /// largest dice count that could ever be optimal. The upper bound is
    /// derived from the point where the expected sum exceeds the maximum score.
    ///
    /// ```text
    /// max_n = floor(2 * (max + sides) / (sides + 1))
    /// ```
    ///
    /// Each successive PMF is computed by convolving the previous one with
    /// the single-die uniform distribution, using an efficient sliding-window
    /// algorithm (see [`Self::convolve_uniform`]).
    #[must_use]
    pub fn precompute(max: u32, sides: u32) -> Self {
        let max_n = (2 * (max + sides) / (sides + 1)).max(max + 1);

        let mut pmf_table: Vec<Vec<f64>> = Vec::with_capacity(max_n as usize + 1);
        pmf_table.push(vec![1.0]);
        pmf_table.push(vec![1.0 / f64::from(sides); sides as usize]);

        for n in 2..=max_n as usize {
            let pmf = &pmf_table[n - 1];
            let convolution = Self::convolve_uniform(pmf, sides);
            pmf_table.push(convolution);
        }

        let data: Box<[f64]> = pmf_table.into_iter().flat_map(Vec::into_iter).collect();
        let offsets: Box<[usize]> = (0..=max_n)
            .map(|n| (n + (sides.saturating_sub(1) * n * n.saturating_sub(1) / 2)) as usize)
            .collect();

        Self {
            data,
            offsets,
            max_n,
        }
    }

    /// Convolve `pmf` with the uniform distribution on {1, …, `sides`}.
    ///
    /// Computes the PMF of `X + U` where `X` has the given `pmf` and `U` is
    /// uniform on a single die. Implemented as a sliding window over a
    /// running sum, giving O(|X|) time regardless of `sides`.
    #[must_use]
    pub fn convolve_uniform(pmf: &[f64], sides: u32) -> Vec<f64> {
        let convolution_len = pmf.len() + sides as usize - 1;
        let mut convolution = Vec::with_capacity(convolution_len);
        let mut running_sum = 0.0;
        for i in 0..convolution_len {
            if i < pmf.len() {
                running_sum += pmf[i];
            }
            if i >= sides as usize {
                running_sum -= pmf[i - sides as usize];
            }
            convolution.push(running_sum / f64::from(sides));
        }
        convolution
    }

    /// Returns the probability `P(sum = total | n)`.
    ///
    /// # Safety
    ///
    /// Caller must ensure `n` ≤ `max_n` and `total` ≥ `n`.
    #[must_use]
    pub fn pmf(&self, n: u32, total: u32) -> f64 {
        let max_n = self.max_n;
        debug_assert!(n <= max_n, "n={n} exceeds max_n={max_n}");
        debug_assert!(total >= n, "total={total} less than n={n}");
        unsafe {
            let offset = *self.offsets.get_unchecked(n as usize);
            let index = offset + (total - n) as usize;
            self.data[index]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {
            assert_approx_eq!($a, $b, 1e-12);
        };
        ($a:expr, $b:expr, $tol:expr) => {{
            let left = $a;
            let right = $b;
            let tol = $tol;
            let diff = (left - right).abs();
            assert!(
                diff <= tol,
                "assertion failed: values are not approximately equal\n  left: {left}\n right: {right}\n  diff: {diff}\n   tol: {tol}"
            );
        }};
    }
    #[test]
    fn pmf_zero_die() {
        let pmf_lookup = PMFLookup::precompute(0, 0);
        assert_eq!(pmf_lookup.pmf(0, 0), 1.0);
    }

    #[test]
    fn pmf_one_die() {
        let pmf_lookup = PMFLookup::precompute(1, 6);
        assert_approx_eq!(pmf_lookup.pmf(1, 1), 1.0 / 6.0);
        assert_approx_eq!(pmf_lookup.pmf(1, 2), 1.0 / 6.0);
        assert_approx_eq!(pmf_lookup.pmf(1, 3), 1.0 / 6.0);
        assert_approx_eq!(pmf_lookup.pmf(1, 4), 1.0 / 6.0);
        assert_approx_eq!(pmf_lookup.pmf(1, 5), 1.0 / 6.0);
        assert_approx_eq!(pmf_lookup.pmf(1, 6), 1.0 / 6.0);
    }

    #[test]
    fn pmf_two_die() {
        let pmf_lookup = PMFLookup::precompute(2, 6);
        assert_approx_eq!(pmf_lookup.pmf(2, 2), 1.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 3), 2.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 4), 3.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 5), 4.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 6), 5.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 7), 6.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 8), 5.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 9), 4.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 10), 3.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 11), 2.0 / 36.0);
        assert_approx_eq!(pmf_lookup.pmf(2, 12), 1.0 / 36.0);
    }

    #[test]
    fn pmf_three_die() {
        let pmf_lookup = PMFLookup::precompute(3, 6);
        assert_approx_eq!(pmf_lookup.pmf(3, 3), 1.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 4), 3.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 5), 6.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 6), 10.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 7), 15.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 8), 21.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 9), 25.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 10), 27.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 11), 27.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 12), 25.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 13), 21.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 14), 15.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 15), 10.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 16), 6.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 17), 3.0 / 216.0);
        assert_approx_eq!(pmf_lookup.pmf(3, 18), 1.0 / 216.0);
    }
}
