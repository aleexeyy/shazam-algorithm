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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_audio_errors() {
        let err = find_spectral_peaks(&[]).unwrap_err();
        matches!(err, FingerprintError::EmptyAudio);
    }

    #[test]
    fn very_short_audio_produces_one_frame() {
        let peaks = find_spectral_peaks(&[0.1]).unwrap();
        assert_eq!(peaks.len(), 1);
    }

    #[test]
    fn frame_count_respects_padding_and_hop() {
        // With FRAME_LENGTH=1024 and HOP_LENGTH=512:
        // len=1025 -> padded_len=1536 -> last_start=512 -> frames=2
        let audio = vec![0.0_f64; FRAME_LENGTH + 1];
        let peaks = find_spectral_peaks(&audio).unwrap();
        assert_eq!(peaks.len(), 2);
    }
}
