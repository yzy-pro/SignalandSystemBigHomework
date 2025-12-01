// 2. FFT 计算模块
// 使用 rustfft 库对音频信号进行快速傅里叶变换

use rustfft::{FftPlanner, num_complex::Complex};
use std::f64::consts::PI;

/// FFT 结果结构
#[derive(Debug, Clone)]
pub struct FftResult {
    /// 频谱复数数据
    pub spectrum: Vec<Complex<f64>>,
    /// 频率轴（Hz）
    pub frequencies: Vec<f64>,
    /// 幅度谱
    pub magnitude: Vec<f64>,
    /// 相位谱
    pub phase: Vec<f64>,
    /// 采样率
    pub sample_rate: f64,
}

impl FftResult {
    /// 计算信号的 FFT
    pub fn compute(samples: &[f64], sample_rate: f64) -> Self {
        let n = samples.len();
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);

        // 将实数信号转换为复数
        let mut buffer: Vec<Complex<f64>> = samples
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // 执行 FFT
        fft.process(&mut buffer);

        // 计算频率轴
        let frequencies: Vec<f64> = (0..n)
            .map(|k| k as f64 * sample_rate / n as f64)
            .collect();

        // 计算幅度谱（归一化）
        let magnitude: Vec<f64> = buffer
            .iter()
            .map(|c| c.norm() / n as f64)
            .collect();

        // 计算相位谱
        let phase: Vec<f64> = buffer
            .iter()
            .map(|c| c.arg())
            .collect();

        println!("FFT 计算完成:");
        println!("  FFT 点数: {}", n);
        println!("  频率分辨率: {:.2} Hz", sample_rate / n as f64);

        FftResult {
            spectrum: buffer,
            frequencies,
            magnitude,
            phase,
            sample_rate,
        }
    }

    /// 执行逆 FFT
    pub fn ifft(spectrum: &[Complex<f64>]) -> Vec<f64> {
        let n = spectrum.len();
        let mut planner = FftPlanner::new();
        let ifft = planner.plan_fft_inverse(n);

        let mut buffer = spectrum.to_vec();
        ifft.process(&mut buffer);

        // 提取实部并归一化
        buffer
            .iter()
            .map(|c| c.re / n as f64)
            .collect()
    }

    /// 获取单边频谱（0 到 Nyquist 频率）
    pub fn get_single_sided(&self) -> (Vec<f64>, Vec<f64>) {
        let nyquist_index = self.frequencies.len() / 2;
        let freqs = self.frequencies[..=nyquist_index].to_vec();
        let mags = self.magnitude[..=nyquist_index].to_vec();
        (freqs, mags)
    }

    /// 获取 dB 刻度的幅度谱
    pub fn get_magnitude_db(&self) -> Vec<f64> {
        self.magnitude
            .iter()
            .map(|&m| {
                if m > 1e-10 {
                    20.0 * m.log10()
                } else {
                    -200.0
                }
            })
            .collect()
    }

    /// 应用窗函数（Hanning 窗）
    pub fn apply_hanning_window(samples: &[f64]) -> Vec<f64> {
        let n = samples.len();
        samples
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let window = 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1) as f64).cos());
                x * window
            })
            .collect()
    }

    /// 应用窗函数（Hamming 窗）
    pub fn apply_hamming_window(samples: &[f64]) -> Vec<f64> {
        let n = samples.len();
        samples
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let window = 0.54 - 0.46 * (2.0 * PI * i as f64 / (n - 1) as f64).cos();
                x * window
            })
            .collect()
    }
}

/// 频谱搬移（循环移位）
pub fn circshift(spectrum: &[Complex<f64>], shift: isize) -> Vec<Complex<f64>> {
    let n = spectrum.len();
    let shift = shift.rem_euclid(n as isize) as usize;
    
    let mut result = vec![Complex::new(0.0, 0.0); n];
    for i in 0..n {
        result[(i + shift) % n] = spectrum[i];
    }
    result
}

/// 计算频域搬移后的和（用于解调）
pub fn frequency_shift_and_add(
    spectrum: &[Complex<f64>],
    shift_hz: f64,
    sample_rate: f64,
) -> Vec<Complex<f64>> {
    let n = spectrum.len();
    let shift_bins = (shift_hz * n as f64 / sample_rate).round() as isize;

    // 正向搬移和负向搬移
    let shifted_pos = circshift(spectrum, shift_bins);
    let shifted_neg = circshift(spectrum, -shift_bins);

    // 相加并除以 2
    shifted_pos
        .iter()
        .zip(shifted_neg.iter())
        .map(|(a, b)| (a + b) / 2.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft() {
        // 测试 FFT 和 IFFT
        let sample_rate = 1000.0;
        let duration = 1.0;
        let n = (sample_rate * duration) as usize;
        
        // 生成测试信号：10 Hz 正弦波
        let samples: Vec<f64> = (0..n)
            .map(|i| (2.0 * PI * 10.0 * i as f64 / sample_rate).sin())
            .collect();

        let fft_result = FftResult::compute(&samples, sample_rate);
        assert_eq!(fft_result.magnitude.len(), n);
        assert_eq!(fft_result.frequencies.len(), n);

        // 测试 IFFT
        let reconstructed = FftResult::ifft(&fft_result.spectrum);
        assert_eq!(reconstructed.len(), n);
        
        // 验证重构误差
        let error: f64 = samples
            .iter()
            .zip(reconstructed.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
            / n as f64;
        assert!(error < 1e-10);
    }

    #[test]
    fn test_circshift() {
        let data: Vec<Complex<f64>> = (0..5)
            .map(|i| Complex::new(i as f64, 0.0))
            .collect();
        
        let shifted = circshift(&data, 2);
        assert_eq!(shifted[0].re, 3.0);
        assert_eq!(shifted[1].re, 4.0);
        assert_eq!(shifted[2].re, 0.0);
    }
}
