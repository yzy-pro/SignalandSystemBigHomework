use num_complex::Complex;
use std::f64::consts::PI;
use crate::butterworth_filter::ButterworthFilter;

/// Frequency response of a filter
pub struct FilterResponse {
    pub frequencies: Vec<f64>,
    pub magnitude: Vec<f64>,
    pub phase: Vec<f64>,
    pub complex_response: Vec<Complex<f64>>,
}

impl FilterResponse {
    /// Compute frequency response of a filter at specified frequency points
    pub fn compute(filter: &ButterworthFilter, sample_rate: f64, num_points: usize) -> Self {
        let mut frequencies = Vec::with_capacity(num_points / 2 + 1);
        let mut magnitude = Vec::with_capacity(num_points / 2 + 1);
        let mut phase = Vec::with_capacity(num_points / 2 + 1);
        let mut complex_response = Vec::with_capacity(num_points / 2 + 1);

        // Generate frequency points from 0 to Nyquist frequency
        for k in 0..=(num_points / 2) {
            let freq = k as f64 * sample_rate / num_points as f64;
            frequencies.push(freq);

            // Compute H(e^(jω)) at this frequency
            let omega = 2.0 * PI * freq / sample_rate;
            let h = frequency_response_at_omega(filter, omega);

            complex_response.push(h);
            magnitude.push(h.norm());
            phase.push(h.arg());
        }

        Self {
            frequencies,
            magnitude,
            phase,
            complex_response,
        }
    }
}

/// Calculate frequency response H(e^(jω)) for a given normalized frequency ω
fn frequency_response_at_omega(filter: &ButterworthFilter, omega: f64) -> Complex<f64> {
    // H(e^(jω)) = B(e^(jω)) / A(e^(jω))
    // where B(e^(jω)) = Σ b[k] * e^(-jωk)
    //       A(e^(jω)) = Σ a[k] * e^(-jωk)

    let mut numerator = Complex::new(0.0, 0.0);
    let mut denominator = Complex::new(0.0, 0.0);

    // Calculate numerator: Σ b[k] * e^(-jωk)
    for (k, &b_k) in filter.b.iter().enumerate() {
        let exp_term = Complex::new(
            (-(k as f64) * omega).cos(),
            (-(k as f64) * omega).sin(),
        );
        numerator += b_k * exp_term;
    }

    // Calculate denominator: Σ a[k] * e^(-jωk)
    for (k, &a_k) in filter.a.iter().enumerate() {
        let exp_term = Complex::new(
            (-(k as f64) * omega).cos(),
            (-(k as f64) * omega).sin(),
        );
        denominator += a_k * exp_term;
    }

    // Return H(e^(jω)) = B(e^(jω)) / A(e^(jω))
    numerator / denominator
}

/// Calculate magnitude in dB: 20 * log10(|H|)
pub fn magnitude_to_db(magnitude: f64) -> f64 {
    20.0 * magnitude.max(1e-10).log10()
}

/// Calculate phase in degrees
pub fn phase_to_degrees(phase_rad: f64) -> f64 {
    phase_rad * 180.0 / PI
}
