// 3. 频谱可视化模块
// 使用 plotters 库绘制频谱图

use plotters::prelude::*;
use std::path::Path;

/// 频谱可视化器
pub struct SpectrumVisualizer;

impl SpectrumVisualizer {
    /// 绘制频谱图（幅度谱）
    pub fn plot_spectrum<P: AsRef<Path>>(
        frequencies: &[f64],
        magnitude: &[f64],
        output_path: P,
        title: &str,
        max_freq: Option<f64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 只显示到指定频率或 Nyquist 频率
        let nyquist = frequencies.last().copied().unwrap_or(0.0) / 2.0;
        let max_f = max_freq.unwrap_or(nyquist);
        
        // 过滤数据点
        let data: Vec<(f64, f64)> = frequencies
            .iter()
            .zip(magnitude.iter())
            .filter(|(&f, _)| f <= max_f)
            .map(|(&f, &m)| (f, m))
            .collect();

        if data.is_empty() {
            return Err("没有数据可以绘制".into());
        }

        // 找出幅度范围
        let max_magnitude = data.iter().map(|(_, m)| m).fold(0.0f64, |a, &b| a.max(b));
        let y_max = max_magnitude * 1.1;

        // 创建绘图区域 - 使用文件路径
        let root = BitMapBackend::new(output_path.as_ref(), (1200, 600))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("Arial", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..max_f, 0.0..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Frequency (Hz)")
            .y_desc("Magnitude")
            .draw()?;

        // 绘制频谱曲线
        chart.draw_series(LineSeries::new(
            data.iter().map(|&(f, m)| (f, m)),
            &BLUE,
        ))?;

        root.present()?;
        println!("频谱图已保存到: {:?}", output_path.as_ref());
        Ok(())
    }

    /// 绘制频谱图（dB 刻度）
    pub fn plot_spectrum_db<P: AsRef<Path>>(
        frequencies: &[f64],
        magnitude_db: &[f64],
        output_path: P,
        title: &str,
        max_freq: Option<f64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let nyquist = frequencies.last().copied().unwrap_or(0.0) / 2.0;
        let max_f = max_freq.unwrap_or(nyquist);
        
        let data: Vec<(f64, f64)> = frequencies
            .iter()
            .zip(magnitude_db.iter())
            .filter(|(&f, _)| f <= max_f)
            .map(|(&f, &m)| (f, m))
            .collect();

        if data.is_empty() {
            return Err("没有数据可以绘制".into());
        }

        let max_db = data.iter().map(|(_, m)| m).fold(-200.0f64, |a, &b| a.max(b));
        let min_db = -100.0;

        let root = BitMapBackend::new(output_path.as_ref(), (1200, 600))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("Arial", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..max_f, min_db..max_db)?;

        chart
            .configure_mesh()
            .x_desc("Frequency (Hz)")
            .y_desc("Magnitude (dB)")
            .draw()?;

        chart.draw_series(LineSeries::new(
            data.iter().map(|&(f, m)| (f, m)),
            &RED,
        ))?;

        root.present()?;
        println!("频谱图（dB）已保存到: {:?}", output_path.as_ref());
        Ok(())
    }

    /// 绘制时域波形
    pub fn plot_waveform<P: AsRef<Path>>(
        samples: &[f64],
        sample_rate: f64,
        output_path: P,
        title: &str,
        max_samples: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 限制显示的采样点数量
        let n = max_samples.unwrap_or(samples.len()).min(samples.len());
        let time: Vec<f64> = (0..n)
            .map(|i| i as f64 / sample_rate)
            .collect();
        
        let data: Vec<(f64, f64)> = time
            .iter()
            .zip(samples.iter())
            .take(n)
            .map(|(&t, &s)| (t, s))
            .collect();

        let max_amplitude = samples.iter().map(|&x| x.abs()).fold(0.0f64, f64::max);
        let y_range = max_amplitude * 1.2;

        let root = BitMapBackend::new(output_path.as_ref(), (1200, 600))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("Arial", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..time[n-1], -y_range..y_range)?;

        chart
            .configure_mesh()
            .x_desc("Time (seconds)")
            .y_desc("Amplitude")
            .draw()?;

        chart.draw_series(LineSeries::new(
            data.iter().map(|&(t, s)| (t, s)),
            &GREEN,
        ))?;

        root.present()?;
        println!("时域波形图已保存到: {:?}", output_path.as_ref());
        Ok(())
    }

    /// 绘制多个频谱对比图
    pub fn plot_spectrum_comparison<P: AsRef<Path>>(
        datasets: Vec<(&[f64], &[f64], &str)>, // (frequencies, magnitude, label)
        output_path: P,
        title: &str,
        max_freq: Option<f64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if datasets.is_empty() {
            return Err("没有数据可以绘制".into());
        }

        let nyquist = datasets[0].0.last().copied().unwrap_or(0.0) / 2.0;
        let max_f = max_freq.unwrap_or(nyquist);

        // 找出最大幅度
        let max_magnitude = datasets
            .iter()
            .flat_map(|(_, mag, _)| mag.iter())
            .copied()
            .fold(0.0f64, f64::max);
        let y_max = max_magnitude * 1.1;

        let root = BitMapBackend::new(output_path.as_ref(), (1200, 600))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("Arial", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..max_f, 0.0..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Frequency (Hz)")
            .y_desc("Magnitude")
            .draw()?;

        let colors = [&BLUE, &RED, &GREEN, &CYAN, &MAGENTA];

        for (idx, (freqs, mags, label)) in datasets.iter().enumerate() {
            let data: Vec<(f64, f64)> = freqs
                .iter()
                .zip(mags.iter())
                .filter(|(&f, _)| f <= max_f)
                .map(|(&f, &m)| (f, m))
                .collect();

            let color = colors[idx % colors.len()];
            chart
                .draw_series(LineSeries::new(data.iter().map(|&(f, m)| (f, m)), color))?
                .label(*label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        root.present()?;
        println!("对比频谱图已保存到: {:?}", output_path.as_ref());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_plot_spectrum() {
        // 生成测试数据
        let n = 1000;
        let sample_rate = 1000.0;
        let frequencies: Vec<f64> = (0..n).map(|i| i as f64 * sample_rate / n as f64).collect();
        
        // 模拟频谱
        let magnitude: Vec<f64> = frequencies
            .iter()
            .map(|&f| {
                if (f - 100.0).abs() < 10.0 {
                    1.0
                } else {
                    0.1 * (-((f - 100.0) / 50.0).powi(2)).exp()
                }
            })
            .collect();

        let result = SpectrumVisualizer::plot_spectrum(
            &frequencies,
            &magnitude,
            "/tmp/test_spectrum.png",
            "测试频谱",
            Some(500.0),
        );
        
        assert!(result.is_ok());
    }
}
