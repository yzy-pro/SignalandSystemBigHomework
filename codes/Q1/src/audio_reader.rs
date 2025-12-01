// 1. 音频文件读取模块
// 负责读取 WAV 文件并提取采样数据、采样率和样本数

use hound::{WavReader, WavSpec};
use std::path::Path;

/// 音频数据结构
#[derive(Debug, Clone)]
pub struct AudioData {
    /// 采样数据（归一化为浮点数）
    pub samples: Vec<f64>,
    /// 采样率 (Hz)
    pub sample_rate: u32,
    /// 样本数
    pub num_samples: usize,
    /// WAV 文件规格
    pub spec: WavSpec,
}

impl AudioData {
    /// 从 WAV 文件读取音频数据
    pub fn from_wav<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reader = WavReader::open(path)?;
        let spec = reader.spec();
        let sample_rate = spec.sample_rate;

        // 读取所有采样点并归一化
        let samples: Vec<f64> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader
                    .samples::<f32>()
                    .map(|s| s.unwrap() as f64)
                    .collect()
            }
            hound::SampleFormat::Int => {
                let max_value = (1 << (spec.bits_per_sample - 1)) as f64;
                reader
                    .samples::<i32>()
                    .map(|s| s.unwrap() as f64 / max_value)
                    .collect()
            }
        };

        let num_samples = samples.len();

        println!("音频文件读取成功:");
        println!("  采样率: {} Hz", sample_rate);
        println!("  样本数: {}", num_samples);
        println!("  位深度: {} bits", spec.bits_per_sample);
        println!("  声道数: {}", spec.channels);
        println!("  时长: {:.2} 秒", num_samples as f64 / sample_rate as f64);

        Ok(AudioData {
            samples,
            sample_rate,
            num_samples,
            spec,
        })
    }

    /// 获取信号时长（秒）
    pub fn duration(&self) -> f64 {
        self.num_samples as f64 / self.sample_rate as f64
    }

    /// 获取单声道数据（如果是立体声则转换为单声道）
    pub fn to_mono(&self) -> Vec<f64> {
        if self.spec.channels == 1 {
            self.samples.clone()
        } else {
            // 立体声转单声道：取平均
            self.samples
                .chunks(self.spec.channels as usize)
                .map(|chunk| chunk.iter().sum::<f64>() / chunk.len() as f64)
                .collect()
        }
    }

    /// 保存为 WAV 文件
    pub fn save_wav<P: AsRef<Path>>(
        &self,
        path: P,
        samples: &[f64],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;

        // 归一化并转换为 i16
        let max_amplitude = samples.iter().map(|&x| x.abs()).fold(0.0f64, f64::max);
        let scale = if max_amplitude > 0.0 {
            32767.0 / max_amplitude
        } else {
            32767.0
        };

        for &sample in samples {
            let sample_i16 = (sample * scale).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        writer.finalize()?;
        println!("音频文件保存成功");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_reader() {
        // 测试读取音频文件
        let result = AudioData::from_wav("../project.wav");
        assert!(result.is_ok());
        
        if let Ok(audio) = result {
            assert!(audio.sample_rate > 0);
            assert!(audio.num_samples > 0);
            assert_eq!(audio.samples.len(), audio.num_samples);
        }
    }
}
