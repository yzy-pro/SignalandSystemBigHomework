// 测试滤波器的实际截止频率
use std::f64::consts::PI;

mod butterworth_filter;
mod filter_response;

use butterworth_filter::ButterworthFilter;

fn main() {
    let sample_rate = 22050.0;
    let designed_cutoff_hp = 3225.1032;
    let designed_cutoff_lp = 4000.0;
    
    println!("=== Testing Filter Cutoff Frequencies ===\n");
    
    // Test high-pass filter
    println!("High-pass filter (designed cutoff = {:.4} Hz)", designed_cutoff_hp);
    let hp_filter = ButterworthFilter::highpass(8, designed_cutoff_hp, sample_rate);
    let actual_cutoff_hp = find_3db_cutoff(&hp_filter, sample_rate, true);
    println!("  Actual -3dB cutoff: {:.4} Hz", actual_cutoff_hp);
    println!("  Error: {:.4} Hz ({:.2}%)\n", 
        actual_cutoff_hp - designed_cutoff_hp,
        (actual_cutoff_hp - designed_cutoff_hp) / designed_cutoff_hp * 100.0);
    
    // Test low-pass filter
    println!("Low-pass filter (designed cutoff = {:.4} Hz)", designed_cutoff_lp);
    let lp_filter = ButterworthFilter::lowpass(8, designed_cutoff_lp, sample_rate);
    let actual_cutoff_lp = find_3db_cutoff(&lp_filter, sample_rate, false);
    println!("  Actual -3dB cutoff: {:.4} Hz", actual_cutoff_lp);
    println!("  Error: {:.4} Hz ({:.2}%)\n", 
        actual_cutoff_lp - designed_cutoff_lp,
        (actual_cutoff_lp - designed_cutoff_lp) / designed_cutoff_lp * 100.0);
}

fn find_3db_cutoff(filter: &ButterworthFilter, sample_rate: f64, is_highpass: bool) -> f64 {
    // 寻找-3dB点 (幅度为1/sqrt(2) ≈ 0.7071)
    let target_magnitude = 1.0 / 2.0_f64.sqrt();
    
    // 在0到奈奎斯特频率之间搜索
    let nyquist = sample_rate / 2.0;
    let num_points = 10000;
    
    let mut best_freq = 0.0;
    let mut best_error = f64::INFINITY;
    
    for i in 0..num_points {
        let freq = i as f64 * nyquist / num_points as f64;
        let omega = 2.0 * PI * freq / sample_rate;
        let h = frequency_response_at_omega(filter, omega);
        let mag = h.norm();
        
        let error = (mag - target_magnitude).abs();
        if error < best_error {
            best_error = error;
            best_freq = freq;
        }
    }
    
    // 精细搜索
    let search_range = nyquist / num_points as f64;
    let fine_points = 1000;
    for i in 0..fine_points {
        let freq = best_freq - search_range / 2.0 + i as f64 * search_range / fine_points as f64;
        if freq < 0.0 || freq > nyquist {
            continue;
        }
        let omega = 2.0 * PI * freq / sample_rate;
        let h = frequency_response_at_omega(filter, omega);
        let mag = h.norm();
        
        let error = (mag - target_magnitude).abs();
        if error < best_error {
            best_error = error;
            best_freq = freq;
        }
    }
    
    best_freq
}

fn frequency_response_at_omega(filter: &ButterworthFilter, omega: f64) -> num_complex::Complex<f64> {
    use num_complex::Complex;
    
    let mut numerator = Complex::new(0.0, 0.0);
    let mut denominator = Complex::new(0.0, 0.0);
    
    for (k, &b_k) in filter.b.iter().enumerate() {
        let exp_term = Complex::new(
            (-(k as f64) * omega).cos(),
            (-(k as f64) * omega).sin(),
        );
        numerator += b_k * exp_term;
    }
    
    for (k, &a_k) in filter.a.iter().enumerate() {
        let exp_term = Complex::new(
            (-(k as f64) * omega).cos(),
            (-(k as f64) * omega).sin(),
        );
        denominator += a_k * exp_term;
    }
    
    numerator / denominator
}
