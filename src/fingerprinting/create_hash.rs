use crate::backend::error::AppError;
use crate::backend::repositories::Repository;
use crate::fingerprinting::constants::{FRAME_LENGTH, HOP_LENGTH, OUTPUT_SAMPLE_RATE};
use crate::fingerprinting::match_song::{MatchConfig, match_song};
use crate::fingerprinting::types::{FingerprintSample, FramePeaks};

const MAX_TARGETS_PER_ANCHOR: usize = 5;
const TARGET_ZONE_FRAMES: usize = 6;

pub fn create_pairs(
    repo: &dyn Repository,
    peaks: &[FramePeaks],
    song_id: u64,
    to_recognize: bool,
) -> Result<u64, AppError> {
    let sample = build_sample(peaks);

    if !to_recognize {
        repo.insert_fingerprints(song_id, &sample.keys, &sample.anchor_times)?;
        return Ok(song_id);
    }

    let matches = repo.get_fingerprints_by_keys(&sample.keys)?;
    let result = match_song(
        &matches,
        &sample.keys,
        &sample.anchor_times,
        MatchConfig::default(),
    )
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(result.song_id)
}

fn build_sample(peaks: &[FramePeaks]) -> FingerprintSample {
    let mut sample = FingerprintSample::default();
    sample.keys.reserve(peaks.len().saturating_mul(16));
    sample.anchor_times.reserve(peaks.len().saturating_mul(16));

    for (anchor_frame_idx, anchor_frame) in peaks.iter().enumerate() {
        if anchor_frame.is_empty() {
            continue;
        }

        let anchor_time = frame_time_s(anchor_frame_idx);
        for anchor_bin in anchor_frame.iter_kept_bins() {
            let mut target_count = 0usize;

            let end = (anchor_frame_idx + 1 + TARGET_ZONE_FRAMES).min(peaks.len());
            for target_frame_idx in (anchor_frame_idx + 1)..end {
                let target_frame = &peaks[target_frame_idx];
                if target_frame.is_empty() {
                    continue;
                }

                let delta_time = frame_time_s(target_frame_idx) - anchor_time;
                for target_bin in target_frame.iter_kept_bins() {
                    if target_count >= MAX_TARGETS_PER_ANCHOR {
                        break;
                    }

                    sample
                        .keys
                        .push(fingerprint_key(anchor_bin, target_bin, delta_time));
                    sample.anchor_times.push(anchor_time);
                    target_count += 1;
                }

                if target_count >= MAX_TARGETS_PER_ANCHOR {
                    break;
                }
            }
        }
    }

    sample
}

fn frame_time_s(frame_idx: usize) -> f32 {
    (HOP_LENGTH * frame_idx) as f32 / (OUTPUT_SAMPLE_RATE as f32)
}

fn fingerprint_key(anchor_bin: u16, target_bin: u16, delta_time_s: f32) -> u64 {
    let norm_anchor = normalize_bin(anchor_bin);
    let norm_target = normalize_bin(target_bin);
    let norm_delta = normalize_delta(delta_time_s);

    (u64::from(norm_anchor)) | (u64::from(norm_target) << 16) | (u64::from(norm_delta) << 32)
}

fn normalize_bin(bin: u16) -> u16 {
    let frame_len = FRAME_LENGTH as u32;
    let clamped = (u32::from(bin)).min(frame_len);
    ((clamped.saturating_mul(65535)) / frame_len) as u16
}

fn normalize_delta(delta_time_s: f32) -> u16 {
    let clamped = delta_time_s.clamp(0.0, 5.0);
    ((clamped / 5.0) * 65535.0).round().clamp(0.0, 65535.0) as u16
}
