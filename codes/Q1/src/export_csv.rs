// 导出频谱分析结果为 CSV 文件

mod audio_reader;
mod fft_processor;
mod frequency_estimator;

use audio_reader::AudioData;
use fft_processor::FftResult;
use frequency_estimator::FrequencyEstimator;
use std::error::Error;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    println!("========================================");
    println!("Q1: 导出频谱分析结果为 CSV");
    println!("========================================\n");

    // 读取音频文件
    println!("读取音频文件...");
    let audio_path = "../project.wav";
    let audio = AudioData::from_wav(audio_path)?;
    
    let samples = audio.to_mono();
    let sample_rate = audio.sample_rate as f64;
    let num_samples = samples.len();
    
    println!("音频信息:");
    println!("  采样率: {} Hz", sample_rate);
    println!("  样本数: {}", num_samples);
    println!("  时长: {:.2} 秒\n", num_samples as f64 / sample_rate);

    // 计算 FFT
    println!("计算 FFT...");
    let fft_result = FftResult::compute(&samples, sample_rate);
    let frequencies = &fft_result.frequencies;
    let magnitude = &fft_result.magnitude;
    let magnitude_db = fft_result.get_magnitude_db();
    
    println!("FFT 点数: {}", num_samples);
    println!("频率分辨率: {:.4} Hz\n", sample_rate / num_samples as f64);

    // 创建输出目录
    std::fs::create_dir_all("output")?;

    // 1. 导出完整频谱数据
    println!("导出完整频谱数据到 output/Q1_spectrum_full.csv...");
    export_spectrum_data(
        frequencies,
        magnitude,
        &magnitude_db,
        "output/Q1_spectrum_full.csv",
        None,
    )?;

    // 2. 导出低频段频谱 (0-4 kHz)
    println!("导出低频段频谱 (0-4 kHz) 到 output/Q1_spectrum_lowfreq.csv...");
    export_spectrum_data(
        frequencies,
        magnitude,
        &magnitude_db,
        "output/Q1_spectrum_lowfreq.csv",
        Some(4000.0),
    )?;

    // 3. 导出中频段频谱 (0-10 kHz)
    println!("导出中频段频谱 (0-10 kHz) 到 output/Q1_spectrum_midfreq.csv...");
    export_spectrum_data(
        frequencies,
        magnitude,
        &magnitude_db,
        "output/Q1_spectrum_midfreq.csv",
        Some(10000.0),
    )?;

    // 4. 导出时域波形数据
    println!("导出时域波形数据到 output/Q1_waveform.csv...");
    export_waveform_data(&samples, sample_rate, "output/Q1_waveform.csv", Some(10000))?;

    // 5. 导出峰值检测结果
    println!("导出峰值检测结果到 output/Q1_peaks.csv...");
    let (f_d, peak_mag, peak_idx) = FrequencyEstimator::estimate_frequency_offset(
        frequencies,
        magnitude,
        (10.0, 10000.0),
        true,
    );
    
    let f_d_refined = FrequencyEstimator::refined_frequency_estimate(
        frequencies,
        magnitude,
        peak_idx,
    );

    let threshold = magnitude[peak_idx] * 0.1;
    let peaks = FrequencyEstimator::find_multiple_peaks(
        frequencies,
        magnitude,
        10,
        20,
        threshold,
    );
    
    export_peaks_data(&peaks, "output/Q1_peaks.csv")?;

    // 6. 导出能量分布数据
    println!("导出能量分布数据到 output/Q1_energy_distribution.csv...");
    let energy_bands = vec![
        (0.0, 500.0),
        (500.0, 1000.0),
        (1000.0, 2000.0),
        (2000.0, 3000.0),
        (3000.0, 4000.0),
        (4000.0, 6000.0),
        (6000.0, 8000.0),
        (8000.0, 10000.0),
        (10000.0, sample_rate / 2.0),
    ];
    export_energy_distribution(magnitude, frequencies, &energy_bands, "output/Q1_energy_distribution.csv")?;

    // 7. 导出分析结果摘要
    println!("导出分析结果摘要到 output/Q1_summary.csv...");
    export_summary(f_d, f_d_refined, peak_mag, sample_rate, num_samples, "output/Q1_summary.csv")?;

    println!("\n========================================");
    println!("导出完成！");
    println!("所有 CSV 文件已保存到 output 目录");
    println!("========================================\n");

    Ok(())
}

/// 导出频谱数据到 CSV
fn export_spectrum_data(
    frequencies: &[f64],
    magnitude: &[f64],
    magnitude_db: &[f64],
    filename: &str,
    max_freq: Option<f64>,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    
    // 写入 CSV 头部
    writeln!(file, "Frequency (Hz),Magnitude,Magnitude (dB)")?;
    
    // 写入数据
    for i in 0..frequencies.len() {
        let freq = frequencies[i];
        
        // 如果设置了最大频率限制，只导出该范围内的数据
        if let Some(max_f) = max_freq {
            if freq > max_f {
                break;
            }
        }
        
        writeln!(file, "{:.4},{:.6},{:.6}", freq, magnitude[i], magnitude_db[i])?;
    }
    
    Ok(())
}

/// 导出时域波形数据到 CSV
fn export_waveform_data(
    samples: &[f64],
    sample_rate: f64,
    filename: &str,
    max_samples: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    
    // 写入 CSV 头部
    writeln!(file, "Time (s),Amplitude")?;
    
    // 确定要导出的样本数
    let num_samples = max_samples.unwrap_or(samples.len()).min(samples.len());
    
    // 写入数据
    for i in 0..num_samples {
        let time = i as f64 / sample_rate;
        writeln!(file, "{:.6},{:.6}", time, samples[i])?;
    }
    
    Ok(())
}

/// 导出峰值检测结果到 CSV
fn export_peaks_data(
    peaks: &[(f64, f64, usize)],
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    
    // 写入 CSV 头部
    writeln!(file, "Peak No.,Frequency (Hz),Magnitude,Index")?;
    
    // 写入数据
    for (i, (freq, mag, idx)) in peaks.iter().enumerate() {
        writeln!(file, "{},{:.4},{:.6},{}", i + 1, freq, mag, idx)?;
    }
    
    Ok(())
}

/// 导出能量分布数据到 CSV
fn export_energy_distribution(
    magnitude: &[f64],
    frequencies: &[f64],
    bands: &[(f64, f64)],
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    
    // 写入 CSV 头部
    writeln!(file, "Band,Start Freq (Hz),End Freq (Hz),Energy,Percentage (%)")?;
    
    // 计算总能量
    let total_energy: f64 = magnitude.iter().map(|&m| m * m).sum();
    
    // 计算每个频段的能量
    for (band_idx, &(f_start, f_end)) in bands.iter().enumerate() {
        let mut band_energy = 0.0;
        
        for (i, &freq) in frequencies.iter().enumerate() {
            if freq >= f_start && freq < f_end {
                band_energy += magnitude[i] * magnitude[i];
            }
        }
        
        let percentage = (band_energy / total_energy) * 100.0;
        writeln!(
            file,
            "{},{:.1},{:.1},{:.6},{:.2}",
            band_idx + 1,
            f_start,
            f_end,
            band_energy,
            percentage
        )?;
    }
    
    Ok(())
}

/// 导出分析结果摘要到 CSV
fn export_summary(
    f_d: f64,
    f_d_refined: f64,
    peak_mag: f64,
    sample_rate: f64,
    num_samples: usize,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    
    // 写入 CSV 头部
    writeln!(file, "Parameter,Value,Unit")?;
    
    // 写入基本信息
    writeln!(file, "Sample Rate,{:.2},Hz", sample_rate)?;
    writeln!(file, "Number of Samples,{},samples", num_samples)?;
    writeln!(file, "Duration,{:.4},s", num_samples as f64 / sample_rate)?;
    writeln!(file, "Frequency Resolution,{:.4},Hz", sample_rate / num_samples as f64)?;
    writeln!(file, "Nyquist Frequency,{:.2},Hz", sample_rate / 2.0)?;
    
    // 写入估计结果
    writeln!(file, "Frequency Offset (Basic),{:.2},Hz", f_d)?;
    writeln!(file, "Frequency Offset (Refined),{:.4},Hz", f_d_refined)?;
    writeln!(file, "Peak Magnitude,{:.6},-", peak_mag)?;
    
    // 写入计算的参数
    writeln!(file, "Baseband Bandwidth,4000.0,Hz")?;
    writeln!(file, "Highpass Cutoff,{:.4},Hz", f_d_refined)?;
    writeln!(file, "Lowpass Cutoff,4000.0,Hz")?;
    
    Ok(())
}
