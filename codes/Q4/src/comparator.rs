use plotters::prelude::*;

pub struct ComparisonResult {
    pub mse: f64,
    pub max_diff: f64,
    pub correlation: f64,
    pub correlation_normalized: f64,
    pub snr_db: f64,
}

/// Compare two signals
pub fn compare_signals(signal1: &[f64], signal2: &[f64]) -> ComparisonResult {
    let n = signal1.len().min(signal2.len());
    
    // Mean Squared Error
    let mse: f64 = (0..n)
        .map(|i| {
            let diff = signal1[i] - signal2[i];
            diff * diff
        })
        .sum::<f64>() / n as f64;
    
    // Maximum absolute difference
    let max_diff = (0..n)
        .map(|i| (signal1[i] - signal2[i]).abs())
        .fold(0.0f64, f64::max);
    
    // Correlation coefficient (original)
    let mean1 = signal1[..n].iter().sum::<f64>() / n as f64;
    let mean2 = signal2[..n].iter().sum::<f64>() / n as f64;
    
    let cov: f64 = (0..n)
        .map(|i| (signal1[i] - mean1) * (signal2[i] - mean2))
        .sum::<f64>() / n as f64;
    
    let var1: f64 = (0..n)
        .map(|i| (signal1[i] - mean1).powi(2))
        .sum::<f64>() / n as f64;
    
    let var2: f64 = (0..n)
        .map(|i| (signal2[i] - mean2).powi(2))
        .sum::<f64>() / n as f64;
    
    let correlation = if var1 > 0.0 && var2 > 0.0 {
        cov / (var1.sqrt() * var2.sqrt())
    } else {
        0.0
    };
    
    // Correlation coefficient with amplitude normalization
    // Normalize both signals to [-1, 1] range based on their max absolute value
    let max_abs1 = signal1[..n].iter().map(|&x| x.abs()).fold(0.0f64, f64::max);
    let max_abs2 = signal2[..n].iter().map(|&x| x.abs()).fold(0.0f64, f64::max);
    
    let correlation_normalized = if max_abs1 > 0.0 && max_abs2 > 0.0 {
        let norm1: Vec<f64> = signal1[..n].iter().map(|&x| x / max_abs1).collect();
        let norm2: Vec<f64> = signal2[..n].iter().map(|&x| x / max_abs2).collect();
        
        let mean_norm1 = norm1.iter().sum::<f64>() / n as f64;
        let mean_norm2 = norm2.iter().sum::<f64>() / n as f64;
        
        let cov_norm: f64 = (0..n)
            .map(|i| (norm1[i] - mean_norm1) * (norm2[i] - mean_norm2))
            .sum::<f64>() / n as f64;
        
        let var_norm1: f64 = (0..n)
            .map(|i| (norm1[i] - mean_norm1).powi(2))
            .sum::<f64>() / n as f64;
        
        let var_norm2: f64 = (0..n)
            .map(|i| (norm2[i] - mean_norm2).powi(2))
            .sum::<f64>() / n as f64;
        
        if var_norm1 > 0.0 && var_norm2 > 0.0 {
            cov_norm / (var_norm1.sqrt() * var_norm2.sqrt())
        } else {
            0.0
        }
    } else {
        0.0
    };
    
    // Signal-to-Noise Ratio (treating difference as noise)
    let signal_power: f64 = signal1[..n].iter().map(|&x| x * x).sum::<f64>() / n as f64;
    let noise_power = mse;
    
    let snr_db = if noise_power > 0.0 {
        10.0 * (signal_power / noise_power).log10()
    } else {
        f64::INFINITY
    };
    
    ComparisonResult {
        mse,
        max_diff,
        correlation,
        correlation_normalized,
        snr_db,
    }
}

/// Save comparison results to file
pub fn save_comparison(result: &ComparisonResult, filename: &str) {
    let mut content = String::new();
    content.push_str("Q4 vs Q3 Comparison Results\n");
    content.push_str("============================\n\n");
    content.push_str(&format!("Mean Squared Error (MSE): {:.6e}\n", result.mse));
    content.push_str(&format!("Root Mean Squared Error (RMSE): {:.6e}\n", result.mse.sqrt()));
    content.push_str(&format!("Maximum absolute difference: {:.6}\n", result.max_diff));
    content.push_str(&format!("Correlation coefficient (original): {:.6}\n", result.correlation));
    content.push_str(&format!("Correlation coefficient (normalized): {:.6}\n", result.correlation_normalized));
    content.push_str(&format!("Signal-to-Noise Ratio: {:.2} dB\n\n", result.snr_db));
    
    content.push_str("Interpretation:\n");
    content.push_str("---------------\n");
    
    // Use normalized correlation for interpretation (more accurate for waveform similarity)
    if result.correlation_normalized > 0.99 {
        content.push_str("✓ Excellent correlation (normalized) - waveforms are nearly identical\n");
    } else if result.correlation_normalized > 0.95 {
        content.push_str("✓ Good correlation (normalized) - waveforms are very similar\n");
    } else if result.correlation_normalized > 0.8 {
        content.push_str("~ Moderate correlation (normalized) - some waveform differences\n");
    } else {
        content.push_str("✗ Low correlation (normalized) - significant waveform differences\n");
    }
    
    content.push_str(&format!("\nNote: Normalized correlation ({:.3}) adjusts for amplitude differences,\n", result.correlation_normalized));
    content.push_str("      providing a better measure of waveform shape similarity.\n");
    content.push_str(&format!("      Original correlation ({:.3}) is affected by both amplitude and shape.\n", result.correlation));
    
    if result.snr_db > 40.0 {
        content.push_str("✓ Excellent SNR - minimal difference\n");
    } else if result.snr_db > 20.0 {
        content.push_str("✓ Good SNR - acceptable difference\n");
    } else {
        content.push_str("~ Low SNR - noticeable difference\n");
    }
    
    content.push_str("\nMethod differences:\n");
    content.push_str("-------------------\n");
    content.push_str("Q3 (Time-domain):\n");
    content.push_str("  - Uses 8th-order Butterworth filters (non-ideal)\n");
    content.push_str("  - IIR filter implementation (Direct Form II)\n");
    content.push_str("  - Non-linear phase response\n");
    content.push_str("  - Gradual transition band\n\n");
    
    content.push_str("Q4 (Frequency-domain):\n");
    content.push_str("  - Uses ideal brick-wall filters\n");
    content.push_str("  - Frequency-domain multiplication\n");
    content.push_str("  - No phase distortion from filters\n");
    content.push_str("  - Sharp cutoff\n");
    
    std::fs::write(filename, content).expect("Failed to save comparison");
}

/// Plot full-time comparison of two signals (all samples)
pub fn plot_full_comparison(signal1: &[f64], signal2: &[f64], filename: &str) {
    let n = signal1.len().min(signal2.len());
    
    let root = BitMapBackend::new(filename, (1600, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    
    let max_val = signal1[..n].iter()
        .chain(signal2[..n].iter())
        .fold(0.0f64, |max, &x| max.max(x.abs()));
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Q4 vs Q3 Signal Comparison (Full Waveform)", ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, -max_val*1.1..max_val*1.1)
        .unwrap();
    
    chart
        .configure_mesh()
        .x_desc("Sample")
        .y_desc("Amplitude")
        .draw()
        .unwrap();
    
    // Plot Q4 signal
    chart
        .draw_series(LineSeries::new(
            (0..n).map(|i| (i, signal1[i])),
            &BLUE,
        ))
        .unwrap()
        .label("Q4 (Frequency-domain)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    // Plot Q3 signal  
    chart
        .draw_series(LineSeries::new(
            (0..n).map(|i| (i, signal2[i])),
            &RED,
        ))
        .unwrap()
        .label("Q3 (Time-domain)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
    
    root.present().unwrap();
    println!("  Saved: {}", filename);
}

/// Plot comparison of two signals (first 2000 samples for detail)
pub fn plot_comparison(signal1: &[f64], signal2: &[f64], filename: &str) {
    let n = signal1.len().min(signal2.len()).min(2000); // Plot first 2000 samples
    
    let root = BitMapBackend::new(filename, (1200, 800)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    
    let max_val = signal1[..n].iter()
        .chain(signal2[..n].iter())
        .fold(0.0f64, |max, &x| max.max(x.abs()));
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Q4 vs Q3 Signal Comparison (Detail View)", ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, -max_val*1.1..max_val*1.1)
        .unwrap();
    
    chart
        .configure_mesh()
        .x_desc("Sample")
        .y_desc("Amplitude")
        .draw()
        .unwrap();
    
    // Plot Q4 signal
    chart
        .draw_series(LineSeries::new(
            (0..n).map(|i| (i, signal1[i])),
            &BLUE,
        ))
        .unwrap()
        .label("Q4 (Frequency-domain)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    
    // Plot Q3 signal
    chart
        .draw_series(LineSeries::new(
            (0..n).map(|i| (i, signal2[i])),
            &RED,
        ))
        .unwrap()
        .label("Q3 (Time-domain)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
    
    root.present().unwrap();
    println!("  Saved: {}", filename);
}
