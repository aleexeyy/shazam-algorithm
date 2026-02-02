use crate::fingerprinting::constants::{FRAME_LENGTH, HOP_LENGTH};
use crate::fingerprinting::error::FingerprintError;
use crate::fingerprinting::make_spectr::FrameAnalyzer;
use crate::fingerprinting::types::FramePeaks;
use rayon::prelude::*;

pub fn find_spectral_peaks(audio: &[f64]) -> Result<Vec<FramePeaks>, FingerprintError> {
    if audio.is_empty() {
        return Err(FingerprintError::EmptyAudio);
    }

    let padded_len = ((audio.len() + HOP_LENGTH - 1) / HOP_LENGTH) * HOP_LENGTH;
    let last_start = padded_len.saturating_sub(FRAME_LENGTH);
    let num_frames = (last_start / HOP_LENGTH) + 1;

    let mut out = vec![FramePeaks::default(); num_frames];
    if num_frames >= 128 {
        out.par_iter_mut().enumerate().for_each_init(
            FrameAnalyzer::new,
            |analyzer, (frame_idx, slot)| {
                let start = frame_idx * HOP_LENGTH;
                *slot = analyzer.analyze_peaks(audio, start);
            },
        );
    } else {
        let mut analyzer = FrameAnalyzer::new();
        for (frame_idx, slot) in out.iter_mut().enumerate() {
            let start = frame_idx * HOP_LENGTH;
            *slot = analyzer.analyze_peaks(audio, start);
        }
    }

    Ok(out)
}
