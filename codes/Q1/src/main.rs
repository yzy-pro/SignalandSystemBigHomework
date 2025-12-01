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
    
    // 通过对称峰值分析确定真实的频率偏差
    println!("\n=== 对称峰值分析 ===");
    println!("检测到的峰值：");
    for (i, (freq, mag, _)) in peaks.iter().enumerate() {
        println!("  峰值 {}: {:.2} Hz (幅度: {:.6})", i + 1, freq, mag);
    }
    
    // 寻找对称峰值对（幅度相近的峰值）
    // 仅在低频区域 (0-5000 Hz) 内搜索，因为频率偏差应该在这个范围内
    let mut symmetric_pairs = Vec::new();
    for i in 0..peaks.len() {
        for j in (i+1)..peaks.len() {
            let (f1, mag1, _) = peaks[i];
            let (f2, mag2, _) = peaks[j];
            
            // 只考虑低频区域的峰值
            if f1 > 5000.0 || f2 > 5000.0 {
                continue;
            }
            
            let mag_ratio = mag1.min(mag2) / mag1.max(mag2);
            // 如果幅度相差小于10%，认为是对称峰值对
            if mag_ratio > 0.9 {
                let axis = (f1 + f2) / 2.0;
                let baseband = (f2 - f1).abs() / 2.0;
                symmetric_pairs.push((f1, f2, axis, mag1, mag2, baseband));
            }
        }
    }
    
    // 选择最佳的对称轴（幅度最大的对称峰值对）
    let f_d_symmetric = if let Some(&(f1, f2, axis, mag1, mag2, baseband)) = symmetric_pairs
        .iter()
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap()) {
        println!("\n找到对称峰值对：");
        let (lower_freq, lower_mag) = if f1 < f2 { (f1, mag1) } else { (f2, mag2) };
        let (upper_freq, upper_mag) = if f1 > f2 { (f1, mag1) } else { (f2, mag2) };
        println!("  下边带峰值: {:.2} Hz (幅度: {:.6})", lower_freq, lower_mag);
        println!("  上边带峰值: {:.2} Hz (幅度: {:.6})", upper_freq, upper_mag);
        println!("  频谱对称轴: {:.2} Hz ← 真实的频率偏差 f_d", axis);
        println!("  基带频率成分: {:.2} Hz", baseband);
        axis
    } else {
        println!("警告：未找到明显的对称峰值对，使用峰值搜索结果");
        f_d_refined
    };

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
    println!("1. 频率偏差估计:");
    println!("   峰值搜索法: {:.2} Hz (单个峰值)", f_d);
    println!("   抛物线插值: {:.4} Hz (精确峰值)", f_d_refined);
    println!("   对称峰值法: {:.2} Hz (频谱对称轴) ← 推荐使用", f_d_symmetric);
    println!();
    println!("   说明：");
    println!("   - 单个峰值 ({:.2} Hz) 反映的是原始信号的能量分布", f_d_refined);
    println!("   - 频谱对称轴 ({:.2} Hz) 才是真实的频率偏差 f_d = f_c - f̃_c", f_d_symmetric);
    println!();
    println!("2. 关于 f̃_c 与 f_c 的大小关系:");
    println!("   - 仅从幅度谱无法唯一确定 f̃_c > f_c 还是 f̃_c < f_c");
    println!("   - 原因: AM 信号的频谱具有共轭对称性");
    println!("   - 无论符号如何，错误解调后的幅度谱都相同");
    println!();
    println!("3. 对解调结果的影响:");
    println!("   - 频率偏差的符号不影响二次解调的效果");
    println!("   - 因为我们使用的是 |f_c - f̃_c| = f_d");
    println!("   - 二次解调时使用 cos(2πf_d·t)，无论符号如何都能正确解调");
    println!();
    println!("4. 所有图形已保存到 output 目录:");
    println!("   - Q1_spectrum_full.png: 全频段频谱");
    println!("   - Q1_spectrum_lowfreq.png: 低频段频谱 (0-4 kHz)");
    println!("   - Q1_spectrum_db.png: dB 刻度频谱");
    println!("   - Q1_waveform.png: 时域波形");
    println!("========================================\n");

    // 保存关键数据供后续使用（使用对称峰值法确定的频率偏差）
    save_results_for_q2(f_d_symmetric, sample_rate)?;

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
