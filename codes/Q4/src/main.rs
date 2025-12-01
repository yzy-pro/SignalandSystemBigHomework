mod audio_reader;
mod ideal_filter;
mod frequency_shifter;
mod spectrum_analyzer;
mod audio_writer;
mod comparator;

use num_complex::Complex;

fn main() {
    println!("Q4: Frequency-Domain Demodulation");
    println!("===================================");

    // Step 1: Read Q1 results to get f_d, f_s, f_B
    println!("\n[Step 1] Reading Q1 results...");
    let (f_d, f_s, f_b) = match read_q1_results() {
        Ok(params) => params,
        Err(e) => {
            eprintln!("Error reading Q1 results: {}", e);
            return;
        }
    };
    println!("  f_d = {:.4} Hz", f_d);
    println!("  f_s = {:.4} Hz", f_s);
    println!("  f_B = {:.4} Hz", f_b);

    // Step 2: Read audio signal
    println!("\n[Step 2] Reading audio signal...");
    let audio_samples = match audio_reader::read_wav("../../工程设计问题-2022/工程设计题15. 调幅信号的解调/project.wav") {
        Ok(samples) => samples,
        Err(e) => {
            eprintln!("Error reading audio: {}", e);
            return;
        }
    };
    let n = audio_samples.len();
    println!("  Number of samples: {}", n);

    // Step 3: Compute FFT of input signal
    println!("\n[Step 3] Computing FFT of input signal...");
    let x_fft = compute_fft(&audio_samples);
    println!("  FFT size: {}", x_fft.len());

    // Step 4: Apply ideal high-pass filter in frequency domain
    println!("\n[Step 4] Applying ideal high-pass filter (fc = {:.4} Hz)...", f_d);
    let xh_fft = ideal_filter::apply_highpass(&x_fft, f_d, f_s, n);
    println!("  High-pass filtering complete");

    // Step 5: Frequency shift (equivalent to carrier multiplication in time domain)
    println!("\n[Step 5] Performing frequency shift (±{:.4} Hz)...", f_d);
    let xb_fft = frequency_shifter::frequency_shift(&xh_fft, f_d, f_s, n);
    println!("  Frequency shift complete");

    // Step 6: Apply ideal low-pass filter
    println!("\n[Step 6] Applying ideal low-pass filter (fc = {:.4} Hz)...", f_b);
    let xl_fft = ideal_filter::apply_lowpass(&xb_fft, f_b, f_s, n);
    println!("  Low-pass filtering complete");

    // Step 7: Inverse FFT to get time-domain signal
    println!("\n[Step 7] Computing IFFT to recover time-domain signal...");
    let mut xl_samples = compute_ifft(&xl_fft);
    println!("  Output samples: {}", xl_samples.len());
    
    // Apply gain compensation (multiply by 2 to match time-domain method)
    for sample in xl_samples.iter_mut() {
        *sample *= 2.0;
    }
    
    let max_val = xl_samples.iter().fold(0.0f64, |max, &x| max.max(x.abs()));
    println!("  Signal max: {:.6}", max_val);

    // Step 8: Create output directory
    std::fs::create_dir_all("output").expect("Failed to create output directory");

    // Step 9: Spectrum analysis for each stage
    println!("\n[Step 8] Performing spectrum analysis...");
    let original_spectrum = compute_magnitude_spectrum(&x_fft, f_s);
    let xh_spectrum = compute_magnitude_spectrum(&xh_fft, f_s);
    let xb_spectrum = compute_magnitude_spectrum(&xb_fft, f_s);
    let xl_spectrum = compute_magnitude_spectrum(&xl_fft, f_s);

    // Step 10: Plot spectra
    println!("\n[Step 9] Plotting spectra...");
    spectrum_analyzer::plot_spectrum(&original_spectrum, "output/Q4_original_spectrum.png", "Original Signal X(f)");
    spectrum_analyzer::plot_spectrum(&xh_spectrum, "output/Q4_xh_spectrum.png", "After Ideal High-Pass X_h(f)");
    spectrum_analyzer::plot_spectrum(&xb_spectrum, "output/Q4_xb_spectrum.png", "After Frequency Shift X_b(f)");
    spectrum_analyzer::plot_spectrum(&xl_spectrum, "output/Q4_xl_spectrum.png", "After Ideal Low-Pass X_l(f) - Demodulated");

    // Step 11: Save demodulated audio
    println!("\n[Step 10] Saving demodulated audio...");
    match audio_writer::write_wav("output/Q4_demodulated.wav", &xl_samples, f_s as u32) {
        Ok(_) => println!("  Saved to: output/Q4_demodulated.wav"),
        Err(e) => eprintln!("  Error saving audio: {}", e),
    }

    // Step 12: Compare with Q3 results
    println!("\n[Step 11] Comparing with Q3 results...");
    if let Ok(q3_samples) = audio_reader::read_wav("../Q3/output/Q3_demodulated.wav") {
        let comparison = comparator::compare_signals(&xl_samples, &q3_samples);
        println!("  Q3 vs Q4 comparison:");
        println!("    MSE: {:.6e}", comparison.mse);
        println!("    Max difference: {:.6}", comparison.max_diff);
        println!("    Correlation (original): {:.6}", comparison.correlation);
        println!("    Correlation (normalized): {:.6}", comparison.correlation_normalized);
        
        // Save comparison results
        comparator::save_comparison(&comparison, "output/Q4_comparison.txt");
        
        // Plot full-time comparison (all samples)
        comparator::plot_full_comparison(&xl_samples, &q3_samples, "output/Q4_vs_Q3_full_comparison.png");
        
        // Plot detailed comparison (first 2000 samples)
        comparator::plot_comparison(&xl_samples, &q3_samples, "output/Q4_vs_Q3_comparison.png");
    } else {
        println!("  Warning: Could not read Q3 results for comparison");
    }

    // Step 13: Save analysis results
    println!("\n[Step 12] Saving analysis results...");
    save_results(&original_spectrum, &xh_spectrum, &xb_spectrum, &xl_spectrum, f_d, f_s, f_b);

    println!("\nQ4 Frequency-Domain Demodulation completed successfully!");
    println!("Output files saved in: codes/Q4/output/");
}

fn read_q1_results() -> Result<(f64, f64, f64), String> {
    let content = std::fs::read_to_string("../Q1/output/Q1_results.txt")
        .map_err(|e| format!("Failed to read Q1 results: {}", e))?;

    let mut f_d = None;
    let mut f_s = None;

    for line in content.lines() {
        if line.contains("频率偏差") || line.contains("f_d") {
            if let Some(value_str) = line.split('=').nth(1) {
                if let Ok(value) = value_str.trim().split_whitespace().next().unwrap_or("0").parse::<f64>() {
                    f_d = Some(value);
                }
            }
        } else if line.contains("采样频率") || line.contains("f_s") {
            if let Some(value_str) = line.split('=').nth(1) {
                if let Ok(value) = value_str.trim().split_whitespace().next().unwrap_or("0").parse::<f64>() {
                    f_s = Some(value);
                }
            }
        }
    }

    let f_d = f_d.ok_or_else(|| "Could not find f_d in Q1 results".to_string())?;
    let f_s = f_s.ok_or_else(|| "Could not find f_s in Q1 results".to_string())?;
    let f_b = 4000.0; // Given in problem statement

    Ok((f_d, f_s, f_b))
}

fn compute_fft(samples: &[f64]) -> Vec<Complex<f64>> {
    use rustfft::FftPlanner;
    
    let mut buffer: Vec<Complex<f64>> = samples
        .iter()
        .map(|&x| Complex::new(x, 0.0))
        .collect();
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buffer.len());
    fft.process(&mut buffer);
    
    buffer
}

fn compute_ifft(spectrum: &[Complex<f64>]) -> Vec<f64> {
    use rustfft::FftPlanner;
    
    let mut buffer = spectrum.to_vec();
    
    let mut planner = FftPlanner::new();
    let ifft = planner.plan_fft_inverse(buffer.len());
    ifft.process(&mut buffer);
    
    // Normalize and extract real part
    let n = buffer.len() as f64;
    buffer.iter().map(|c| c.re / n).collect()
}

fn compute_magnitude_spectrum(spectrum: &[Complex<f64>], f_s: f64) -> Vec<(f64, f64)> {
    let n = spectrum.len();
    let df = f_s / n as f64;
    
    (0..n/2)
        .map(|i| {
            let freq = i as f64 * df;
            let magnitude = spectrum[i].norm() / n as f64;
            (freq, magnitude)
        })
        .collect()
}

fn save_results(
    original: &[(f64, f64)],
    xh: &[(f64, f64)],
    xb: &[(f64, f64)],
    xl: &[(f64, f64)],
    f_d: f64,
    f_s: f64,
    f_b: f64,
) {
    let mut content = String::new();
    content.push_str("Q4 Frequency-Domain Demodulation Results\n");
    content.push_str("==========================================\n\n");
    content.push_str(&format!("Carrier frequency: f_d = {:.4} Hz\n", f_d));
    content.push_str(&format!("Sampling frequency: f_s = {:.4} Hz\n", f_s));
    content.push_str(&format!("Baseband bandwidth: f_B = {:.4} Hz\n\n", f_b));

    // Spectral peaks for each stage
    content.push_str("Spectral Analysis:\n");
    content.push_str("------------------\n");
    
    // Original signal peak
    let orig_peak = original.iter()
        .filter(|(f, _)| *f > 1000.0)
        .max_by(|(_, mag1), (_, mag2)| mag1.partial_cmp(mag2).unwrap())
        .unwrap();
    content.push_str(&format!("Original signal X(f) peak: f = {:.2} Hz, magnitude = {:.6}\n", 
        orig_peak.0, orig_peak.1));
    
    // After high-pass
    let xh_peak = xh.iter()
        .filter(|(f, _)| *f > f_d)
        .max_by(|(_, mag1), (_, mag2)| mag1.partial_cmp(mag2).unwrap())
        .unwrap();
    content.push_str(&format!("After ideal high-pass X_h(f) peak: f = {:.2} Hz, magnitude = {:.6}\n", 
        xh_peak.0, xh_peak.1));
    
    // After frequency shift
    let xb_peak = xb.iter()
        .filter(|(f, _)| *f > 10.0 && *f < 5000.0)
        .max_by(|(_, mag1), (_, mag2)| mag1.partial_cmp(mag2).unwrap())
        .unwrap();
    content.push_str(&format!("After frequency shift X_b(f) peak: f = {:.2} Hz, magnitude = {:.6}\n", 
        xb_peak.0, xb_peak.1));
    
    // Demodulated signal peak
    let xl_peak = xl.iter()
        .filter(|(f, _)| *f > 10.0 && *f < f_b)
        .max_by(|(_, mag1), (_, mag2)| mag1.partial_cmp(mag2).unwrap())
        .unwrap();
    content.push_str(&format!("Demodulated signal X_l(f) peak: f = {:.2} Hz, magnitude = {:.6}\n", 
        xl_peak.0, xl_peak.1));
    
    // Energy in baseband
    let energy_orig: f64 = original.iter()
        .filter(|(f, _)| *f < f_b)
        .map(|(_, m)| m * m)
        .sum();
    let energy_demod: f64 = xl.iter()
        .filter(|(f, _)| *f < f_b)
        .map(|(_, m)| m * m)
        .sum();
    
    content.push_str(&format!("\nEnergy analysis (0-{:.0} Hz band):\n", f_b));
    content.push_str(&format!("  Original signal energy: {:.6e}\n", energy_orig));
    content.push_str(&format!("  Demodulated signal energy: {:.6e}\n", energy_demod));
    
    content.push_str(&format!("\nMethod characteristics:\n"));
    content.push_str("  - Uses ideal filters (brick-wall response)\n");
    content.push_str("  - Frequency-domain processing (no time-domain convolution)\n");
    content.push_str("  - Perfect frequency selectivity\n");
    content.push_str("  - No phase distortion from filters\n");

    std::fs::write("output/Q4_results.txt", content).expect("Failed to save results");
    println!("  Saved to: output/Q4_results.txt");
}
