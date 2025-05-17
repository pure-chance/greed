use crate::pmf::{pmf_exact, pmf_normal_approximation};

pub fn precompute(error: f64, min_n_cap: u16) -> Vec<(u16, u16)> {
    let mut s = 2;
    let mut results = vec![(0, u16::MAX), (1, u16::MAX)];
    let mut min_n = min_n_cap;

    loop {
        let mut low = 1u16;
        let mut high = min_n;

        while low < high {
            let n = low + (high - low) / 2;

            // Check error across all possible totals for this n and s
            let mut sum_diff = 0.0;
            let mut valid = true;
            for total in n..=n * s {
                let exact = pmf_exact(total, n, s);
                let approx = pmf_normal_approximation(total, n, s);
                let diff = (exact - approx).abs();
                if !diff.is_finite() {
                    valid = false;
                    break;
                }
                sum_diff += diff;
            }
            let avg_diff = if valid {
                sum_diff / (n * s - n + 1) as f64
            } else {
                f64::INFINITY
            };

            if avg_diff <= error {
                min_n = n;
                high = n - 1;
            } else {
                low = n + 1;
            }
        }

        results.push((s, min_n));
        println!("Precomputed s = {s}, min_n = {min_n}");

        if min_n <= 1 {
            break;
        }
        s += 1;
    }
    results
}
