use crate::fingerprinting::match_song::{MatchConfig, match_song};
use crate::fingerprinting::types::FramePeaks;

#[test]
fn frame_peaks_iter_kept_bins_respects_mask() {
    let peaks = FramePeaks {
        bins: [10, 20, 30, 40, 50, 60],
        keep_mask: 0b0010_0101,
    };

    let got: Vec<u16> = peaks.iter_kept_bins().collect();
    assert_eq!(got, vec![10, 30, 60]);
}

#[test]
fn match_song_picks_best_song() {
    // Sample has two hashes at times 0 and 1.
    let sample_keys = vec![111u64, 222u64];
    let sample_times = vec![0.0f32, 1.0f32];

    // Song 1 matches both hashes with a consistent +1.0s offset.
    // Song 2 matches one hash only.
    let matches = vec![
        (111u64, 1u64, 1.0f32),
        (222u64, 1u64, 2.0f32),
        (111u64, 2u64, 10.0f32),
    ];

    let result = match match_song(
        &matches,
        &sample_keys,
        &sample_times,
        MatchConfig::default(),
    ) {
        Ok(v) => v,
        Err(e) => panic!("match should succeed: {e}"),
    };
    assert_eq!(result.song_id, 1);
    assert!(result.confidence > 0.0);
}
