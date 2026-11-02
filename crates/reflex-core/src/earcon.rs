//! Embedded low-latency PCM audio earcons.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarconKind {
    Success,
    Warning,
    ConfirmationChime,
    Shutter,
    Error,
}

pub const SAMPLE_RATE: u32 = 48000;

impl EarconKind {
    /// Generate 16-bit 48kHz mono PCM audio samples for this earcon
    pub fn pcm_samples(&self) -> Vec<i16> {
        match self {
            EarconKind::Success => {
                // 15ms 440Hz click with exponential decay
                let duration_samples = (SAMPLE_RATE as f32 * 0.015) as usize;
                let mut samples = Vec::with_capacity(duration_samples);
                for i in 0..duration_samples {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let decay = (-150.0 * t).exp();
                    let wave = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
                    let val = (wave * decay * 18000.0) as i16;
                    samples.push(val);
                }
                samples
            }
            EarconKind::Warning => {
                // 40ms double-thud (160Hz pulse, 10ms gap, 160Hz pulse)
                let duration_samples = (SAMPLE_RATE as f32 * 0.040) as usize;
                let mut samples = Vec::with_capacity(duration_samples);
                let pulse_samples = (SAMPLE_RATE as f32 * 0.012) as usize;
                let gap_samples = (SAMPLE_RATE as f32 * 0.010) as usize;

                for i in 0..duration_samples {
                    let val = if i < pulse_samples {
                        let t = i as f32 / SAMPLE_RATE as f32;
                        let decay = (-80.0 * t).exp();
                        ((2.0 * std::f32::consts::PI * 160.0 * t).sin() * decay * 20000.0) as i16
                    } else if i < pulse_samples + gap_samples {
                        0
                    } else {
                        let local_i = i - pulse_samples - gap_samples;
                        let t = local_i as f32 / SAMPLE_RATE as f32;
                        let decay = (-80.0 * t).exp();
                        ((2.0 * std::f32::consts::PI * 160.0 * t).sin() * decay * 20000.0) as i16
                    };
                    samples.push(val);
                }
                samples
            }
            EarconKind::ConfirmationChime => {
                // 100ms ascending two-tone chime (523Hz for 50ms, then 659Hz for 50ms)
                let duration_samples = (SAMPLE_RATE as f32 * 0.100) as usize;
                let half = duration_samples / 2;
                let mut samples = Vec::with_capacity(duration_samples);

                for i in 0..duration_samples {
                    let (freq, t) = if i < half {
                        (523.25, i as f32 / SAMPLE_RATE as f32)
                    } else {
                        (659.25, (i - half) as f32 / SAMPLE_RATE as f32)
                    };
                    let decay = (-25.0 * t).exp();
                    let wave = (2.0 * std::f32::consts::PI * freq * t).sin();
                    samples.push((wave * decay * 16000.0) as i16);
                }
                samples
            }
            EarconKind::Shutter => {
                // 30ms camera shutter click (double click noise burst)
                let duration_samples = (SAMPLE_RATE as f32 * 0.030) as usize;
                let mut samples = Vec::with_capacity(duration_samples);
                for i in 0..duration_samples {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let decay = (-100.0 * t).exp();
                    // High-frequency burst
                    let wave = (2.0 * std::f32::consts::PI * 1200.0 * t).sin();
                    samples.push((wave * decay * 18000.0) as i16);
                }
                samples
            }
            EarconKind::Error => {
                // 60ms low buzz at 120Hz
                let duration_samples = (SAMPLE_RATE as f32 * 0.060) as usize;
                let mut samples = Vec::with_capacity(duration_samples);
                for i in 0..duration_samples {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let wave = (2.0 * std::f32::consts::PI * 120.0 * t).sin();
                    let decay = (-30.0 * t).exp();
                    samples.push((wave * decay * 20000.0) as i16);
                }
                samples
            }
        }
    }

    /// Return raw bytes (little-endian 16-bit PCM)
    pub fn pcm_bytes(&self) -> Vec<u8> {
        let samples = self.pcm_samples();
        let mut bytes = Vec::with_capacity(samples.len() * 2);
        for s in samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        bytes
    }
}
