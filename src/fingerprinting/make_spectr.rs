use crate::fingerprinting::constants::FRAME_LENGTH;
use crate::fingerprinting::types::FramePeaks;
use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

const BANDS: [(u16, u16); 6] = [
    (4, 6),   // ~30-40 Hz
    (6, 11),  // ~40-80 Hz
    (11, 16), // ~80-120 Hz
    (16, 24), // ~120-180 Hz
    (24, 40), // ~180-300 Hz
    (40, 65), // ~300-500 Hz
];

pub struct FrameAnalyzer {
    window: Vec<f64>,
    fft: Arc<dyn Fft<f64>>,
    buffer: Vec<Complex<f64>>,
}

impl FrameAnalyzer {
    pub fn new() -> Self {
        let window = hann_window(FRAME_LENGTH);

        let mut planner = FftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(FRAME_LENGTH);

        let buffer = vec![Complex::new(0.0, 0.0); FRAME_LENGTH];

        Self {
            window,
            fft,
            buffer,
        }
    }

    pub fn analyze_peaks(&mut self, audio: &[f64], start: usize) -> FramePeaks {
        self.fill_windowed(audio, start);
        self.fft.process(&mut self.buffer);

        let mut max_bins = [0u16; 6];
        let mut max_vals = [f64::NEG_INFINITY; 6];

        for (band_idx, (start_bin, end_bin)) in BANDS.iter().enumerate() {
            let mut best_bin = *start_bin;
            let mut best_val = f64::NEG_INFINITY;

            for bin in *start_bin..*end_bin {
                let bin_usize = usize::from(bin);
                if bin_usize >= self.buffer.len() {
                    break;
                }
                let mag = self.buffer[bin_usize].norm_sqr();
                if mag > best_val {
                    best_val = mag;
                    best_bin = bin;
                }
            }

            max_bins[band_idx] = best_bin;
            max_vals[band_idx] = best_val;
        }

        let avg = max_vals.iter().sum::<f64>() / (max_vals.len() as f64);
        let mut keep_mask = 0u8;
        for (i, &val) in max_vals.iter().enumerate() {
            if val > avg {
                keep_mask |= 1u8 << i;
            }
        }

        FramePeaks {
            bins: max_bins,
            keep_mask,
        }
    }

    fn fill_windowed(&mut self, audio: &[f64], start: usize) {
        for i in 0..FRAME_LENGTH {
            let sample = audio.get(start + i).copied().unwrap_or(0.0);
            self.buffer[i] = Complex::new(sample * self.window[i], 0.0);
        }
    }
}

fn hann_window(len: usize) -> Vec<f64> {
    if len == 0 {
        return Vec::new();
    }

    if len == 1 {
        return vec![1.0];
    }

    let denom = (len - 1) as f64;
    (0..len)
        .map(|n| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * n as f64 / denom).cos()))
        .collect()
}
