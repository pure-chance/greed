use rustfft::{num_complex::Complex, FftPlanner};

/// Convolve two real-valued PMFs using FFT
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
