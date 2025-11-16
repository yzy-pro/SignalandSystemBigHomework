use std::f64::consts::PI;
use num_complex::Complex;

pub struct ButterworthFilter {
    pub b: Vec<f64>,
    pub a: Vec<f64>,
    pub order: usize,
    pub cutoff: f64,
    pub sample_rate: f64,
    pub filter_type: FilterType,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    Lowpass,
    Highpass,
}

impl ButterworthFilter {
    pub fn lowpass(order: usize, cutoff: f64, sample_rate: f64) -> Self {
        let (b, a) = design_butterworth_digital_lowpass(order, cutoff, sample_rate);
        Self { b, a, order, cutoff, sample_rate, filter_type: FilterType::Lowpass }
    }

    pub fn highpass(order: usize, cutoff: f64, sample_rate: f64) -> Self {
        let (b, a) = design_butterworth_digital_highpass(order, cutoff, sample_rate);
        Self { b, a, order, cutoff, sample_rate, filter_type: FilterType::Highpass }
    }
}

fn design_butterworth_digital_lowpass(order: usize, cutoff: f64, fs: f64) -> (Vec<f64>, Vec<f64>) {
    // Pre-warp the cutoff frequency to compensate for bilinear transform distortion
    let wc = 2.0 * fs * (PI * cutoff / fs).tan();
    let poles = butterworth_analog_poles(order);
    let scaled_poles: Vec<_> = poles.iter().map(|(re, im)| (re * wc, im * wc)).collect();
    bilinear_transform_cascade(&scaled_poles, fs)
}

fn design_butterworth_digital_highpass(order: usize, cutoff: f64, fs: f64) -> (Vec<f64>, Vec<f64>) {
    // Spectral inversion method: H_HP(z) = H_LP(-z)
    // The cutoff frequency is mirrored about fs/4
    // To get highpass with cutoff fc_hp, design lowpass with cutoff (fs/2 - fc_hp)
    let lp_cutoff = fs / 2.0 - cutoff;
    
    // Design a lowpass filter at the mirrored cutoff frequency
    let (b_lp, a_lp) = design_butterworth_digital_lowpass(order, lp_cutoff, fs);
    
    // Convert lowpass to highpass using spectral inversion: H_HP(z) = H_LP(-z)
    // This means alternating the signs of coefficients with odd indices
    let mut b_hp = b_lp.clone();
    let mut a_hp = a_lp.clone();
    
    for (i, val) in b_hp.iter_mut().enumerate() {
        if i % 2 == 1 {
            *val = -*val;
        }
    }
    
    for (i, val) in a_hp.iter_mut().enumerate() {
        if i % 2 == 1 {
            *val = -*val;
        }
    }
    
    (b_hp, a_hp)
}

fn butterworth_analog_poles(order: usize) -> Vec<(f64, f64)> {
    (0..order).map(|k| {
        let theta = PI * (2.0 * k as f64 + order as f64 + 1.0) / (2.0 * order as f64);
        (theta.cos(), theta.sin())
    }).collect()
}

fn bilinear_transform_cascade(poles: &[(f64, f64)], fs: f64) -> (Vec<f64>, Vec<f64>) {
    let t = 1.0 / fs;
    let mut b_total = vec![1.0];
    let mut a_total = vec![1.0];
    let mut i = 0;
    while i < poles.len() {
        let (pr1, pi1) = poles[i];
        if pi1.abs() < 1e-10 {
            let denom = 2.0 - pr1 * t;
            let z_pole = (2.0 + pr1 * t) / denom;
            b_total = convolve(&b_total, &vec![1.0, 1.0]);
            a_total = convolve(&a_total, &vec![1.0, -z_pole]);
            i += 1;
        } else {
            if i + 1 < poles.len() {
                let b_section = vec![1.0, 2.0, 1.0];
                let denom_re = 2.0 - pr1 * t;
                let denom_im = -pi1 * t;
                let denom_mag_sq = denom_re * denom_re + denom_im * denom_im;
                let z1_re = ((2.0 + pr1 * t) * denom_re + pi1 * t * denom_im) / denom_mag_sq;
                let z1_im = ((pi1 * t) * denom_re - (2.0 + pr1 * t) * denom_im) / denom_mag_sq;
                let a1 = -2.0 * z1_re;
                let a2 = z1_re * z1_re + z1_im * z1_im;
                let a_section = vec![1.0, a1, a2];
                b_total = convolve(&b_total, &b_section);
                a_total = convolve(&a_total, &a_section);
                i += 2;
            } else {
                i += 1;
            }
        }
    }
    let a0 = a_total[0];
    for i in 0..a_total.len() { a_total[i] /= a0; }
    for i in 0..b_total.len() { b_total[i] /= a0; }
    let b_sum: f64 = b_total.iter().sum();
    let a_sum: f64 = a_total.iter().sum();
    let gain = a_sum / b_sum;
    for i in 0..b_total.len() { b_total[i] *= gain; }
    (b_total, a_total)
}

fn convolve(a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; a.len() + b.len() - 1];
    for (i, &a_val) in a.iter().enumerate() {
        for (j, &b_val) in b.iter().enumerate() {
            result[i + j] += a_val * b_val;
        }
    }
    result
}

fn lowpass_to_highpass(b_lp: &[f64], a_lp: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut b_hp = b_lp.to_vec();
    let mut a_hp = a_lp.to_vec();
    for (i, val) in b_hp.iter_mut().enumerate() {
        if i % 2 == 1 { *val = -*val; }
    }
    for (i, val) in a_hp.iter_mut().enumerate() {
        if i % 2 == 1 { *val = -*val; }
    }
    (b_hp, a_hp)
}
