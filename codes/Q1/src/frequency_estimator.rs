// 4. 频率偏差估计模块
// 通过分析频谱找出频率偏差 f_d

/// 频率偏差估计器
pub struct FrequencyEstimator;

impl FrequencyEstimator {
    /// 估计频率偏差 f_d
    /// 
    /// # 参数
    /// - frequencies: 频率轴
    /// - magnitude: 幅度谱
    /// - search_range: 搜索范围 (Hz)，例如 (10.0, 10000.0)
    /// - exclude_dc: 是否排除直流分量
    /// 
    /// # 返回
    /// (peak_frequency, peak_magnitude, peak_index)
    pub fn estimate_frequency_offset(
        frequencies: &[f64],
        magnitude: &[f64],
        search_range: (f64, f64),
        exclude_dc: bool,
    ) -> (f64, f64, usize) {
        let (min_freq, max_freq) = search_range;
        
        // 在指定范围内搜索峰值
        let mut peak_magnitude = 0.0;
        let mut peak_index = 0;
        let mut peak_frequency = 0.0;

        let start_idx = if exclude_dc { 1 } else { 0 };

        for (i, (&freq, &mag)) in frequencies
            .iter()
            .zip(magnitude.iter())
            .enumerate()
            .skip(start_idx)
        {
            if freq >= min_freq && freq <= max_freq && mag > peak_magnitude {
                peak_magnitude = mag;
                peak_index = i;
                peak_frequency = freq;
            }
        }

        println!("频率偏差估计结果:");
        println!("  峰值频率 f_d = {:.2} Hz", peak_frequency);
        println!("  峰值幅度 = {:.6}", peak_magnitude);
        println!("  峰值索引 = {}", peak_index);

        (peak_frequency, peak_magnitude, peak_index)
    }

    /// 精确估计频率（使用抛物线插值）
    pub fn refined_frequency_estimate(
        frequencies: &[f64],
        magnitude: &[f64],
        peak_index: usize,
    ) -> f64 {
        if peak_index == 0 || peak_index >= magnitude.len() - 1 {
            return frequencies[peak_index];
        }

        // 使用三点抛物线插值
        let y1 = magnitude[peak_index - 1];
        let y2 = magnitude[peak_index];
        let y3 = magnitude[peak_index + 1];

        // 抛物线顶点位置
        let delta = 0.5 * (y1 - y3) / (y1 - 2.0 * y2 + y3);
        
        let freq_resolution = if frequencies.len() > 1 {
            frequencies[1] - frequencies[0]
        } else {
            1.0
        };

        let refined_freq = frequencies[peak_index] + delta * freq_resolution;

        println!("精确频率估计:");
        println!("  原始峰值频率: {:.2} Hz", frequencies[peak_index]);
        println!("  精确频率: {:.4} Hz", refined_freq);

        refined_freq
    }

    /// 寻找多个峰值
    pub fn find_multiple_peaks(
        frequencies: &[f64],
        magnitude: &[f64],
        num_peaks: usize,
        min_distance: usize,
        threshold: f64,
    ) -> Vec<(f64, f64, usize)> {
        let mut peaks = Vec::new();
        let n = magnitude.len();

        // 找出所有局部最大值
        for i in 1..n-1 {
            if magnitude[i] > magnitude[i-1] 
                && magnitude[i] > magnitude[i+1] 
                && magnitude[i] > threshold 
            {
                peaks.push((frequencies[i], magnitude[i], i));
            }
        }

        // 按幅度降序排序
        peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 移除距离太近的峰值
        let mut filtered_peaks = Vec::new();
        for peak in peaks {
            let is_far_enough = filtered_peaks
                .iter()
                .all(|(_, _, idx)| (*idx as isize - peak.2 as isize).abs() >= min_distance as isize);
            
            if is_far_enough {
                filtered_peaks.push(peak);
                if filtered_peaks.len() >= num_peaks {
                    break;
                }
            }
        }

        println!("找到 {} 个峰值:", filtered_peaks.len());
        for (i, (freq, mag, idx)) in filtered_peaks.iter().enumerate() {
            println!("  峰值 {}: 频率 = {:.2} Hz, 幅度 = {:.6}, 索引 = {}", 
                     i + 1, freq, mag, idx);
        }

        filtered_peaks
    }

    /// 计算信号的能量分布
    pub fn compute_energy_distribution(
        magnitude: &[f64],
        frequencies: &[f64],
        bands: &[(f64, f64)], // 频带范围
    ) -> Vec<(String, f64)> {
        let total_energy: f64 = magnitude.iter().map(|&m| m * m).sum();
        
        let mut band_energies = Vec::new();
        
        for (low, high) in bands {
            let energy: f64 = frequencies
                .iter()
                .zip(magnitude.iter())
                .filter(|(&f, _)| f >= *low && f <= *high)
                .map(|(_, &m)| m * m)
                .sum();
            
            let percentage = (energy / total_energy) * 100.0;
            band_energies.push((
                format!("{:.0}-{:.0} Hz", low, high),
                percentage,
            ));
        }

        println!("\n能量分布:");
        for (band, percent) in &band_energies {
            println!("  {}: {:.2}%", band, percent);
        }

        band_energies
    }

    /// 判断 f_c_tilde 与 f_c 的大小关系
    /// 
    /// 注意：仅从频谱的对称性很难直接判断，通常需要相位信息或其他先验知识
    pub fn analyze_frequency_relationship(
        frequencies: &[f64],
        magnitude: &[f64],
        f_d: f64,
    ) -> String {
        println!("\n频率关系分析:");
        println!("  估计的频率偏差 f_d = {:.2} Hz", f_d);
        println!("  仅从幅度谱无法唯一确定 f_c_tilde > f_c 还是 f_c_tilde < f_c");
        println!("  原因：频谱的对称性使得两种情况产生相同的幅度谱");
        println!("  需要：相位信息、时域分析或其他先验知识来确定符号");
        
        String::from("无法仅从幅度谱确定频率偏差的符号")
    }

    /// 计算信噪比（SNR）估计
    pub fn estimate_snr(
        magnitude: &[f64],
        signal_band: (usize, usize),
        noise_band: (usize, usize),
    ) -> f64 {
        let signal_power: f64 = magnitude[signal_band.0..signal_band.1]
            .iter()
            .map(|&m| m * m)
            .sum::<f64>() / (signal_band.1 - signal_band.0) as f64;

        let noise_power: f64 = magnitude[noise_band.0..noise_band.1]
            .iter()
            .map(|&m| m * m)
            .sum::<f64>() / (noise_band.1 - noise_band.0) as f64;

        let snr_db = if noise_power > 0.0 {
            10.0 * (signal_power / noise_power).log10()
        } else {
            f64::INFINITY
        };

        println!("\n信噪比估计:");
        println!("  信号功率: {:.6}", signal_power);
        println!("  噪声功率: {:.6}", noise_power);
        println!("  SNR: {:.2} dB", snr_db);

        snr_db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_frequency_estimation() {
        // 生成测试信号：包含 100 Hz 的峰值
        let n = 1000;
        let sample_rate = 1000.0;
        let frequencies: Vec<f64> = (0..n).map(|i| i as f64 * sample_rate / n as f64).collect();
        
        let magnitude: Vec<f64> = frequencies
            .iter()
            .map(|&f| {
                if (f - 100.0).abs() < 1.0 {
                    1.0
                } else {
                    0.01
                }
            })
            .collect();

        let (peak_freq, peak_mag, peak_idx) = FrequencyEstimator::estimate_frequency_offset(
            &frequencies,
            &magnitude,
            (10.0, 500.0),
            true,
        );

        assert!((peak_freq - 100.0).abs() < 2.0);
        assert!(peak_mag > 0.5);
        assert!(peak_idx > 0);
    }

    #[test]
    fn test_multiple_peaks() {
        let n = 1000;
        let sample_rate = 1000.0;
        let frequencies: Vec<f64> = (0..n).map(|i| i as f64 * sample_rate / n as f64).collect();
        
        // 生成包含两个峰值的信号
        let magnitude: Vec<f64> = frequencies
            .iter()
            .map(|&f| {
                let peak1 = (-((f - 100.0) / 10.0).powi(2)).exp();
                let peak2 = 0.8 * (-((f - 300.0) / 10.0).powi(2)).exp();
                peak1 + peak2
            })
            .collect();

        let peaks = FrequencyEstimator::find_multiple_peaks(
            &frequencies,
            &magnitude,
            2,
            20,
            0.1,
        );

        assert_eq!(peaks.len(), 2);
    }
}
