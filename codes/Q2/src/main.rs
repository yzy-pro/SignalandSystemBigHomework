mod butterworth_filter;
mod filter_response;
mod response_visualizer;

use std::fs;
use std::path::Path;

fn main() {
    println!("=== Q2: Butterworth Filter Design ===\n");

    // Read parameters from Q1 results
    let q1_results_path = "../Q1/output/Q1_results.txt";
    let (sample_rate, f_d, f_b) = read_q1_results(q1_results_path);

    println!("Parameters from Q1:");
    println!("  Sample Rate: {} Hz", sample_rate);
    println!("  Frequency Offset (f_d): {:.4} Hz", f_d);
    println!("  Signal Bandwidth (f_B): {} Hz", f_b);
    println!();

    // Design 8th-order Butterworth filters
    let order = 8;
    println!("Designing 8th-order Butterworth filters...");

    // High-pass filter with cutoff frequency f_d
    println!("  - High-pass filter (cutoff = {:.4} Hz)", f_d);
    let highpass = butterworth_filter::ButterworthFilter::highpass(order, f_d, sample_rate);

    // Low-pass filter with cutoff frequency f_B
    println!("  - Low-pass filter (cutoff = {} Hz)", f_b);
    let lowpass = butterworth_filter::ButterworthFilter::lowpass(order, f_b, sample_rate);

    println!("\nHigh-pass filter coefficients:");
    println!("  b (numerator): {:?}", &highpass.b[..5.min(highpass.b.len())]);
    println!("  a (denominator): {:?}", &highpass.a[..5.min(highpass.a.len())]);

    println!("\nLow-pass filter coefficients:");
    println!("  b (numerator): {:?}", &lowpass.b[..5.min(lowpass.b.len())]);
    println!("  a (denominator): {:?}", &lowpass.a[..5.min(lowpass.a.len())]);

    // Calculate frequency response at the same frequency points as Q1
    let num_points = 31265; // Same as Q1 audio samples
    println!("\nCalculating frequency responses ({} points)...", num_points);

    let hp_response = filter_response::FilterResponse::compute(&highpass, sample_rate, num_points);
    let lp_response = filter_response::FilterResponse::compute(&lowpass, sample_rate, num_points);

    // Create output directory
    let output_dir = "output";
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // Plot frequency responses
    println!("\nGenerating plots...");

    // High-pass filter magnitude response
    response_visualizer::plot_magnitude_response(
        &hp_response.frequencies,
        &hp_response.magnitude,
        &format!("{}/Q2_highpass_magnitude.png", output_dir),
        "High-pass Filter Magnitude Response",
        Some(10000.0),
    ).expect("Failed to plot high-pass magnitude");

    // High-pass filter magnitude response in dB
    response_visualizer::plot_magnitude_response_db(
        &hp_response.frequencies,
        &hp_response.magnitude,
        &format!("{}/Q2_highpass_magnitude_db.png", output_dir),
        "High-pass Filter Magnitude Response (dB)",
        Some(10000.0),
    ).expect("Failed to plot high-pass magnitude dB");

    // High-pass filter phase response
    response_visualizer::plot_phase_response(
        &hp_response.frequencies,
        &hp_response.phase,
        &format!("{}/Q2_highpass_phase.png", output_dir),
        "High-pass Filter Phase Response",
        Some(10000.0),
    ).expect("Failed to plot high-pass phase");

    // Low-pass filter magnitude response
    response_visualizer::plot_magnitude_response(
        &lp_response.frequencies,
        &lp_response.magnitude,
        &format!("{}/Q2_lowpass_magnitude.png", output_dir),
        "Low-pass Filter Magnitude Response",
        Some(10000.0),
    ).expect("Failed to plot low-pass magnitude");

    // Low-pass filter magnitude response in dB
    response_visualizer::plot_magnitude_response_db(
        &lp_response.frequencies,
        &lp_response.magnitude,
        &format!("{}/Q2_lowpass_magnitude_db.png", output_dir),
        "Low-pass Filter Magnitude Response (dB)",
        Some(10000.0),
    ).expect("Failed to plot low-pass magnitude dB");

    // Low-pass filter phase response
    response_visualizer::plot_phase_response(
        &lp_response.frequencies,
        &lp_response.phase,
        &format!("{}/Q2_lowpass_phase.png", output_dir),
        "Low-pass Filter Phase Response",
        Some(10000.0),
    ).expect("Failed to plot low-pass phase");

    // Combined magnitude plot
    response_visualizer::plot_combined_magnitude(
        &hp_response.frequencies,
        &hp_response.magnitude,
        &lp_response.magnitude,
        &format!("{}/Q2_combined_magnitude.png", output_dir),
        "Combined Filter Magnitude Responses",
        Some(10000.0),
    ).expect("Failed to plot combined magnitude");

    // Save filter coefficients
    save_filter_coefficients(&highpass, &lowpass, &format!("{}/Q2_filter_coefficients.txt", output_dir));

    // Save frequency response data
    save_frequency_response(&hp_response, &lp_response, &format!("{}/Q2_frequency_response.txt", output_dir));

    println!("\nAll results saved to '{}/' directory", output_dir);
    println!("\nQ2 completed successfully!");
}

fn read_q1_results(path: &str) -> (f64, f64, f64) {
    let content = fs::read_to_string(path)
        .expect("Failed to read Q1 results file");

    let mut sample_rate = 22050.0;
    let mut f_d = 3225.0;
    let mut f_b = 4000.0;

    for line in content.lines() {
        // Parse: "频率偏差 f_d = 3225.1032 Hz"
        if line.contains("频率偏差") && line.contains("f_d") {
            if let Some(value_str) = line.split('=').nth(1) {
                if let Some(num_str) = value_str.trim().split_whitespace().next() {
                    f_d = num_str.parse().unwrap_or(3225.0);
                }
            }
        }
        // Parse: "采样率 f_s = 22050.00 Hz"
        else if line.contains("采样率") && line.contains("f_s") {
            if let Some(value_str) = line.split('=').nth(1) {
                if let Some(num_str) = value_str.trim().split_whitespace().next() {
                    sample_rate = num_str.parse().unwrap_or(22050.0);
                }
            }
        }
        // Parse: "基带带宽 f_B = 4000 Hz"
        else if line.contains("基带带宽") && line.contains("f_B") {
            if let Some(value_str) = line.split('=').nth(1) {
                if let Some(num_str) = value_str.trim().split_whitespace().next() {
                    f_b = num_str.parse().unwrap_or(4000.0);
                }
            }
        }
    }

    (sample_rate, f_d, f_b)
}

fn save_filter_coefficients(highpass: &butterworth_filter::ButterworthFilter, 
                            lowpass: &butterworth_filter::ButterworthFilter,
                            path: &str) {
    let mut content = String::new();
    content.push_str("=== Q2: Filter Coefficients ===\n\n");

    content.push_str("High-pass Filter (8th-order Butterworth):\n");
    content.push_str(&format!("Cutoff Frequency: {:.4} Hz\n", highpass.cutoff));
    content.push_str(&format!("Sample Rate: {} Hz\n", highpass.sample_rate));
    content.push_str("\nNumerator Coefficients (b):\n");
    for (i, coef) in highpass.b.iter().enumerate() {
        content.push_str(&format!("  b[{}] = {:.15e}\n", i, coef));
    }
    content.push_str("\nDenominator Coefficients (a):\n");
    for (i, coef) in highpass.a.iter().enumerate() {
        content.push_str(&format!("  a[{}] = {:.15e}\n", i, coef));
    }

    content.push_str("\n\nLow-pass Filter (8th-order Butterworth):\n");
    content.push_str(&format!("Cutoff Frequency: {:.4} Hz\n", lowpass.cutoff));
    content.push_str(&format!("Sample Rate: {} Hz\n", lowpass.sample_rate));
    content.push_str("\nNumerator Coefficients (b):\n");
    for (i, coef) in lowpass.b.iter().enumerate() {
        content.push_str(&format!("  b[{}] = {:.15e}\n", i, coef));
    }
    content.push_str("\nDenominator Coefficients (a):\n");
    for (i, coef) in lowpass.a.iter().enumerate() {
        content.push_str(&format!("  a[{}] = {:.15e}\n", i, coef));
    }

    fs::write(path, content).expect("Failed to write filter coefficients");
}

fn save_frequency_response(hp_response: &filter_response::FilterResponse,
                          lp_response: &filter_response::FilterResponse,
                          path: &str) {
    let mut content = String::new();
    content.push_str("=== Q2: Frequency Response Data ===\n\n");

    content.push_str("High-pass Filter:\n");
    content.push_str(&format!("Number of frequency points: {}\n", hp_response.frequencies.len()));
    content.push_str(&format!("Frequency range: 0 - {:.2} Hz\n", hp_response.frequencies.last().unwrap_or(&0.0)));
    content.push_str(&format!("Maximum magnitude: {:.6}\n", hp_response.magnitude.iter().cloned().fold(0./0., f64::max)));
    content.push_str(&format!("Minimum magnitude: {:.6}\n", hp_response.magnitude.iter().cloned().fold(f64::INFINITY, f64::min)));

    content.push_str("\nLow-pass Filter:\n");
    content.push_str(&format!("Number of frequency points: {}\n", lp_response.frequencies.len()));
    content.push_str(&format!("Frequency range: 0 - {:.2} Hz\n", lp_response.frequencies.last().unwrap_or(&0.0)));
    content.push_str(&format!("Maximum magnitude: {:.6}\n", lp_response.magnitude.iter().cloned().fold(0./0., f64::max)));
    content.push_str(&format!("Minimum magnitude: {:.6}\n", lp_response.magnitude.iter().cloned().fold(f64::INFINITY, f64::min)));

    fs::write(path, content).expect("Failed to write frequency response data");
}
