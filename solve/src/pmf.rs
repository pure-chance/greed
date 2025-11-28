use std::ops::{Index, IndexMut};

/// Lookup table for dice roll probability mass functions.
///
/// Precomputes and stores PMFs for all dice counts up to a maximum, enabling
/// O(1) lookup of P(sum = k | n dice). This is the performance-critical
/// component of the solver, as PMF lookups occur millions of times during
/// policy computation.
#[derive(Debug, Clone)]
pub struct PMFLookup {
    /// Flat array containing all PMF data.
    data: Box<[f64]>,
    /// Starting offsets for each n-dice PMF.
    offsets: Box<[usize]>,
    /// Maximum number of dice.
    max_n: u32,
}

impl Default for PMFLookup {
    fn default() -> Self {
        Self {
            data: Box::new([]),
            offsets: Box::new([]),
            max_n: 0,
        }
    }
}

impl PMFLookup {
    /// Precompute all required PMFs for the given game parameters.
    ///
    /// Generates PMFs for 0 to `max_n` dice, where `max_n` is determined by the
    /// largest number of dice that could be strategically relevant. Uses FFT
    /// convolution for efficient computation and creates optimized lookup
    /// tables.
    #[must_use]
    pub fn precompute(max: u32, sides: u32) -> Self {
        let max_n = (2 * (max + sides) / (sides + 1)).max(max + 1);

        let mut pmf_table: Vec<Vec<f64>> = Vec::with_capacity(max_n as usize + 1);
        pmf_table.push(vec![1.0]);
        pmf_table.push(vec![1.0 / f64::from(sides); sides as usize]);

        for n in 2..=max_n as usize {
            let pmf = &pmf_table[n - 1];
            let convolution = Self::sliding_window_convolution(pmf, sides as usize);
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

    /// Returns the convolution of a given PMF with the uniform PMF of a single
    /// die with given number of `sides`.
    ///
    /// This is implemented using a sliding window approach for performance.
    #[must_use]
    pub fn sliding_window_convolution(pmf: &[f64], sides: usize) -> Vec<f64> {
        let mut convolution = Vec::with_capacity(pmf.len() + sides - 1);
        let mut running_sum = 0.0;
        for i in 0..(pmf.len() + sides - 1) {
            if i < pmf.len() {
                running_sum += pmf[i];
            }
            if i >= sides {
                running_sum -= pmf[i - sides];
            }
            #[allow(clippy::cast_precision_loss)]
            convolution.push(running_sum / sides as f64);
        }
        convolution
    }
}

impl Index<(u32, u32)> for PMFLookup {
    type Output = f64;

    /// Returns a reference to the PMF value P(sum = total | n dice).
    ///
    /// # Safety
    ///
    /// Caller must ensure `n` ≤ `max_n` and `total` ≥ `n`.
    #[inline]
    fn index(&self, (n, total): (u32, u32)) -> &Self::Output {
        debug_assert!(n <= self.max_n, "n={} exceeds max_n={}", n, self.max_n);
        debug_assert!(total >= n, "total={total} less than n={n}");
        unsafe {
            let offset = *self.offsets.get_unchecked(n as usize);
            let index = offset + (total - n) as usize;
            &self.data[index]
        }
    }
}

impl IndexMut<(u32, u32)> for PMFLookup {
    /// Returns a mutable reference to the PMF value P(sum = total | n dice).
    ///
    /// # Safety
    ///
    /// Caller must ensure `n` ≤ `max_n` and `total` ≥ `n`.
    fn index_mut(&mut self, (n, total): (u32, u32)) -> &mut Self::Output {
        debug_assert!(n <= self.max_n, "n={} exceeds max_n={}", n, self.max_n);
        debug_assert!(total >= n, "total={total} less than n={n}");
        unsafe {
            let offset = *self.offsets.get_unchecked(n as usize);
            let index = offset + (total - n) as usize;
            &mut self.data[index]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_pmfs() {
        let pmf = PMFLookup::precompute(10, 6);
        assert_eq!(pmf[(0, 0)], 1.0);
        assert_eq!(pmf[(1, 1)], 1.0 / 6.0);
        assert_eq!(pmf[(2, 1)], 1.0 / 36.0);
    }
}
