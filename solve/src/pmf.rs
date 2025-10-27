use rustfft::{FftPlanner, num_complex::Complex};

/// Optimized lookup table for dice roll probability mass functions.
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
    max_n: u16,
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
    pub fn precompute(max: u16, sides: u16) -> Self {
        let max_n = (2 * (max + sides) / (sides + 1)).max(max + 1);
        let dice_pmf = vec![1.0 / f64::from(sides); sides as usize];

        // First pass: compute individual PMFs to determine total size
        let mut temp_pmfs: Vec<Vec<f64>> = Vec::with_capacity((max_n + 1) as usize);
        temp_pmfs.push(vec![1.0]); // n=0 case

        for n in 1..=max_n {
            temp_pmfs.push(Self::fft_convolve(&temp_pmfs[(n - 1) as usize], &dice_pmf));
        }

        // Validate PMFs sum to 1.0
        for (n, pmf) in temp_pmfs.iter().enumerate() {
            if n > 0 {
                let sum: f64 = pmf.iter().sum();
                debug_assert!(
                    (sum - 1.0).abs() < 1e-10,
                    "PMF for {n} dice doesn't sum to 1.0: {sum}",
                );
            }
        }

        // Second pass: flatten into single array with offset table
        let total_size: usize = temp_pmfs.iter().map(Vec::len).sum();
        let mut data = Vec::with_capacity(total_size);
        let mut offsets = Vec::with_capacity((max_n + 1) as usize);

        for pmf in &temp_pmfs {
            offsets.push(data.len());
            data.extend_from_slice(pmf);
        }

        Self {
            data: data.into_boxed_slice(),
            offsets: offsets.into_boxed_slice(),
            max_n,
        }
    }
    /// Convolve two real-valued PMFs using FFT.
    #[must_use]
    pub fn fft_convolve(a: &[f64], b: &[f64]) -> Vec<f64> {
        let size = (a.len() + b.len()).next_power_of_two();
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(size);
        let ifft = planner.plan_fft_inverse(size);

        let mut fa: Vec<Complex<f64>> = a.iter().map(|&x| Complex::new(x, 0.0)).collect();
        fa.resize(size, Complex::new(0.0, 0.0));
        let mut fb: Vec<Complex<f64>> = b.iter().map(|&x| Complex::new(x, 0.0)).collect();
        fb.resize(size, Complex::new(0.0, 0.0));

        fft.process(&mut fa);
        fft.process(&mut fb);

        for (x, y) in fa.iter_mut().zip(fb.iter()) {
            *x *= *y;
        }

        ifft.process(&mut fa);
        fa.truncate(a.len() + b.len() - 1);
        fa.iter().map(|x| (x.re / size as f64).max(0.0)).collect()
    }
    /// Fast lookup of PMF value P(sum = total | n dice).
    ///
    /// Optimized for hot path usage with caching for small n values and unsafe
    /// memory access. Use this in performance-critical code where bounds are
    /// guaranteed.
    ///
    /// # Safety
    ///
    /// Caller must ensure `n` ≤ `max_n` and `total` ≥ `n`.
    #[must_use]
    #[inline]
    pub fn lookup(&self, n: u16, total: u16) -> f64 {
        debug_assert!(n <= self.max_n, "n={} exceeds max_n={}", n, self.max_n);
        debug_assert!(total >= n, "total={total} less than n={n}");

        unsafe {
            let offset = *self.offsets.get_unchecked(n as usize);
            let index = offset + (total - n) as usize;
            *self.data.get_unchecked(index)
        }
    }
}
