// Q1 主程序：频谱分析与频率偏差估计
// 整合四个模块完成完整的分析流程

mod audio_reader;
mod fft_processor;
mod spectrum_visualizer;
mod frequency_estimator;

use audio_reader::AudioData;
use fft_processor::FftResult;
use spectrum_visualizer::SpectrumVisualizer;
use frequency_estimator::FrequencyEstimator;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("========================================");
    println!("Q1: 频谱分析与频率偏差估计");
    println!("========================================\n");

    // ===== 步骤 1: 音频文件读取 =====
    println!("步骤 1: 读取音频文件...");
    let audio_path = "../project.wav";
    let audio = AudioData::from_wav(audio_path)?;
    
    // 转换为单声道（如果需要）
    let samples = audio.to_mono();
    let sample_rate = audio.sample_rate as f64;
    let num_samples = samples.len();
    
    println!("\n音频信息:");
    println!("  采样率 f_s = {} Hz", sample_rate);
    println!("  样本数 N = {}", num_samples);
    println!("  时长 = {:.2} 秒\n", num_samples as f64 / sample_rate);

    // ===== 步骤 2: FFT 计算 =====
    println!("步骤 2: 计算 FFT...");
    let fft_result = FftResult::compute(&samples, sample_rate);
    let frequencies = &fft_result.frequencies;
    let magnitude = &fft_result.magnitude;
    let magnitude_db = fft_result.get_magnitude_db();
    
    println!("  FFT 点数: {}", num_samples);
    println!("  频率分辨率: {:.4} Hz\n", sample_rate / num_samples as f64);

    // ===== 步骤 3: 频谱可视化 =====
    println!("步骤 3: 绘制频谱图...");
    
    // 绘制全频谱
    SpectrumVisualizer::plot_spectrum(
        frequencies,
        magnitude,
        "output/Q1_spectrum_full.png",
        "Spectrum of Misdemodulated Signal (Full)",
        Some(sample_rate / 2.0),
    )?;

    // 绘制低频段频谱（0-10 kHz）
    SpectrumVisualizer::plot_spectrum(
        frequencies,
        magnitude,
        "output/Q1_spectrum_lowfreq.png",
        "Spectrum of Misdemodulated Signal (0-4 kHz)",
        Some(4000.0),
    )?;

    // 绘制 dB 刻度的频谱
    SpectrumVisualizer::plot_spectrum_db(
        frequencies,
        &magnitude_db,
        "output/Q1_spectrum_db.png",
        "Spectrum of Misdemodulated Signal (dB scale)",
        Some(10000.0),
    )?;

    // // 绘制时域波形（前 0.1 秒）
    // let samples_to_plot = (sample_rate * 0.1) as usize;
    // SpectrumVisualizer::plot_waveform(
    //     &samples,
    //     sample_rate,
    //     "output/Q1_waveform.png",
    //     "Waveform of Misdemodulated Signal (First 0.1s)",
    //     Some(samples_to_plot),
    // )?;
    // 绘制时域波形
    let audio_duration = num_samples as f64 / sample_rate;
    let samples_to_plot = (sample_rate * audio_duration) as usize;
    SpectrumVisualizer::plot_waveform(
        &samples,
        sample_rate,
        "output/Q1_waveform.png",
        "Waveform of Misdemodulated Signal",
        Some(samples_to_plot),
    )?;

    println!();

    // ===== 步骤 4: 频率偏差估计 =====
    println!("步骤 4: 估计频率偏差 (First 0.1s)f_d...\n");
    
    // 基本频率估计（排除直流，搜索 10 Hz 到 10 kHz）
    let (f_d, peak_mag, peak_idx) = FrequencyEstimator::estimate_frequency_offset(
        frequencies,
        magnitude,
        (10.0, 10000.0),
        true, // 排除直流分量
    );

    // 精确频率估计（使用抛物线插值）
    let f_d_refined = FrequencyEstimator::refined_frequency_estimate(
        frequencies,
        magnitude,
        peak_idx,
    );

    // 寻找多个峰值
    println!();
    let threshold = magnitude[peak_idx] * 0.1; // 设置阈值为主峰的 10%
    let peaks = FrequencyEstimator::find_multiple_peaks(
        frequencies,
        magnitude,
        5, // 最多找 5 个峰值
        20, // 最小间隔 20 个采样点
        threshold,
    );

    // 计算能量分布
    let energy_bands = vec![
        (0.0, 1000.0),
        (1000.0, 4000.0),
        (4000.0, 8000.0),
        (8000.0, sample_rate / 2.0),
    ];
    FrequencyEstimator::compute_energy_distribution(
        magnitude,
        frequencies,
        &energy_bands,
    );

    // 分析频率关系
    FrequencyEstimator::analyze_frequency_relationship(
        frequencies,
        magnitude,
        f_d_refined,
    );

    // ===== 结果总结 =====
    println!("\n========================================");
    println!("分析结果总结");
    println!("========================================");
    println!("1. 估计的频率偏差:");
    println!("   f_d ≈ {:.2} Hz (基本估计)", f_d);
    println!("   f_d ≈ {:.4} Hz (精确估计)", f_d_refined);
    println!();
    println!("2. 关于 f_c_tilde 与 f_c 的大小关系:");
    println!("   - 仅从幅度谱无法唯一确定 f_c_tilde > f_c 还是 f_c_tilde < f_c");
    println!("   - 原因: AM 信号的频谱具有对称性");
    println!("   - 无论 f_c_tilde > f_c 还是 f_c_tilde < f_c，");
    println!("     错误解调后的信号频谱形态相同");
    println!();
    println!("3. 对解调结果的影响:");
    println!("   - 频率偏差的符号不影响二次解调的效果");
    println!("   - 因为我们使用的是 |f_c_tilde - f_c| = f_d");
    println!("   - 二次解调时使用 cos(2πf_d·t)，无论符号如何都能正确解调");
    println!();
    println!("4. 所有图形已保存到 output 目录:");
    println!("   - Q1_spectrum_full.png: 全频段频谱");
    println!("   - Q1_spectrum_lowfreq.png: 低频段频谱 (0-10 kHz)");
    println!("   - Q1_spectrum_db.png: dB 刻度频谱");
    println!("   - Q1_waveform.png: 时域波形");
    println!("========================================\n");

    // 保存关键数据供后续使用
    save_results_for_q2(f_d_refined, sample_rate)?;

    Ok(())
}

/// 保存结果供 Q2 使用
fn save_results_for_q2(f_d: f64, sample_rate: f64) -> Result<(), Box<dyn Error>> {
    use std::fs;
    use std::io::Write;

    fs::create_dir_all("output")?;
    let mut file = fs::File::create("output/Q1_results.txt")?;
    
    writeln!(file, "Q1 分析结果")?;
    writeln!(file, "===========")?;
    writeln!(file, "频率偏差 f_d = {:.4} Hz", f_d)?;
    writeln!(file, "采样率 f_s = {:.2} Hz", sample_rate)?;
    writeln!(file, "基带带宽 f_B = 4000 Hz")?;
    
    println!("结果已保存到 output/Q1_results.txt");
    
    Ok(())
}
