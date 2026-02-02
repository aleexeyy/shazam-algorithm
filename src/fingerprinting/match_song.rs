use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub struct MatchConfig {
    pub precision_factor: f32,
    pub offset_bin_size: i32,
    pub min_match_threshold: u32,
    pub min_confidence: f32,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            precision_factor: 1000.0,
            offset_bin_size: 5,
            min_match_threshold: 2,
            min_confidence: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MatchResult {
    pub confidence: f32,
    pub song_id: u64,
}

#[derive(Debug, Error)]
pub enum MatchError {
    #[error("sample keys/times length mismatch")]
    SampleMismatch,
}

pub fn match_song(
    matches: &[(u64, u64, f32)],
    sample_keys: &[u64],
    sample_times: &[f32],
    config: MatchConfig,
) -> Result<MatchResult, MatchError> {
    if sample_keys.len() != sample_times.len() {
        return Err(MatchError::SampleMismatch);
    }

    if matches.is_empty() || sample_keys.is_empty() {
        return Ok(MatchResult::default());
    }

    let mut sample_by_hash: HashMap<u64, Vec<f32>> = HashMap::new();
    for (&hash, &t) in sample_keys.iter().zip(sample_times.iter()) {
        sample_by_hash.entry(hash).or_default().push(t);
    }

    let mut hash_frequency: HashMap<u64, u32> = HashMap::new();
    let mut song_row_counts: HashMap<u64, u32> = HashMap::new();

    for &(hash, song_id, _) in matches {
        *hash_frequency.entry(hash).or_insert(0) += 1;
        *song_row_counts.entry(song_id).or_insert(0) += 1;
    }

    let mut offset_counts: HashMap<(u64, i32), u32> = HashMap::new();

    for &(hash, song_id, song_time) in matches {
        let Some(sample_ts) = sample_by_hash.get(&hash) else {
            continue;
        };

        let freq = hash_frequency.get(&hash).copied().unwrap_or(1).max(1);
        let weight = 1.0 / (freq as f32);
        let weight_count = (weight * 100.0).round().max(1.0) as u32;

        for &sample_time in sample_ts {
            let raw_offset = (song_time - sample_time) * config.precision_factor;
            let offset_bin = (raw_offset as i32 / config.offset_bin_size) * config.offset_bin_size;
            *offset_counts.entry((song_id, offset_bin)).or_insert(0) += weight_count;
        }
    }

    let mut best = MatchResult::default();
    let mut best_weighted_matches = 0u32;

    for ((song_id, _offset), weighted_matches) in offset_counts {
        if weighted_matches < config.min_match_threshold {
            continue;
        }

        let denom = song_row_counts.get(&song_id).copied().unwrap_or(0);
        if denom == 0 {
            continue;
        }

        let confidence = (weighted_matches as f32 / denom as f32) * 100.0;
        if confidence < config.min_confidence {
            continue;
        }

        if weighted_matches > best_weighted_matches {
            best_weighted_matches = weighted_matches;
            best = MatchResult {
                confidence,
                song_id,
            };
        }
    }

    Ok(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatch_lengths_errors() {
        let matches: Vec<(u64, u64, f32)> = Vec::new();
        let err = match_song(&matches, &[1, 2], &[0.0], MatchConfig::default()).unwrap_err();
        assert!(matches!(err, MatchError::SampleMismatch));
    }

    #[test]
    fn empty_inputs_return_default() {
        let matches: Vec<(u64, u64, f32)> = Vec::new();
        let result = match_song(&matches, &[], &[], MatchConfig::default()).unwrap();
        assert_eq!(result, MatchResult::default());
    }

    #[test]
    fn chooses_song_with_strongest_consistent_offset() {
        let sample_keys = vec![111u64, 222u64, 333u64];
        let sample_times = vec![0.0f32, 1.0f32, 2.0f32];

        let matches = vec![
            // Song 1 aligns with +1s offset on all keys.
            (111, 1, 1.0),
            (222, 1, 2.0),
            (333, 1, 3.0),
            // Song 2 only aligns on one key.
            (111, 2, 10.0),
        ];

        let result = match_song(
            &matches,
            &sample_keys,
            &sample_times,
            MatchConfig::default(),
        )
        .unwrap();
        assert_eq!(result.song_id, 1);
        assert!(result.confidence > 0.0);
    }
}
