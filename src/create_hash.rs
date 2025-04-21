use crate::database_interaction::*;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::constants::*;
use crate::match_song::match_song;

fn hash_function(anchor_f: f64, target_f: f64, delta_time: f64) -> u64 {

    
    let norm_anchor = (anchor_f.min(OUTPUT_SAMPLE_RATE) / OUTPUT_SAMPLE_RATE * 65535.0) as u64 & 0xFFFF;
    let norm_target = (target_f.min(OUTPUT_SAMPLE_RATE) / OUTPUT_SAMPLE_RATE * 65535.0) as u64 & 0xFFFF;
    let norm_delta = (delta_time.min(5.0) / 5.0 * 65535.0) as u64 & 0xFFFF;

    // Step 2: Create a tuple of the normalized values
    let fingerprint = (norm_anchor, norm_target, norm_delta);
    
    // Step 3: Use Rust's built-in high-quality hashing
    let mut hasher = DefaultHasher::new();
    fingerprint.hash(&mut hasher);
    hasher.finish()
}

//index - sample index in spectrogram
fn convert_index_to_frequency(index : usize) -> f64 {
    // sample_rate / frame_length * index_in_spectrogram
    return (OUTPUT_SAMPLE_RATE / (FRAME_LENGTH as f64)) * (index as f64);
}
// i - frame index, j - sample index in spectrogram
fn convert_index_to_time(i: usize, j: usize) -> f64 {
    return (HOP_LENGTH * i + j) as f64 / OUTPUT_SAMPLE_RATE;
}

fn search_target_zone(peaks : &Vec<Vec<usize>>, frame : usize) -> Result<Vec<(f64, f64)>, Box<dyn std::error::Error>> {

    let mut target_peaks: Vec<(f64, f64)> = Vec::new();

    for target_i in frame+1..frame+7 {
        
        if target_i >= peaks.len() { continue }
        if peaks[target_i].len() == 0 { continue }
        

        for target_j in 0..peaks[target_i].len() {
            let target_peak_frequency = convert_index_to_frequency(peaks[target_i][target_j]);
            let target_peak_time = convert_index_to_time(target_i, peaks[target_i][target_j]);
            target_peaks.push((target_peak_frequency, target_peak_time));

        }

    }

    Ok(target_peaks)
}

pub fn create_pairs(peaks: &Vec<Vec<usize>>, song_id: u64, to_recognize : bool, conn: &mut mysql::PooledConn) -> Result<u64, Box<dyn std::error::Error>> {
    // target zone: 6 frames, all frequencies

    let mut pairs_counter = 0;
    let file = File::create("../log/database_inserts.txt")?;
    let mut writer = BufWriter::new(file);
    let max_targets_per_anchor = 5;
    let mut keys: Vec<u64> = Vec::new();
    let mut values: Vec<f64> = Vec::new();
    for i in 0..peaks.len() {

        for j in 0..peaks[i].len() {

            let anchor = peaks[i][j];
            let anchor_peak_frequency = convert_index_to_frequency(anchor);
            let anchor_peak_time = convert_index_to_time(i, anchor);

            let target_peaks = search_target_zone(peaks, i)?;

            if target_peaks.is_empty() { continue }

            let mut target_count = 0;
            for target_peak in target_peaks {

                if target_count >= max_targets_per_anchor {
                    break;
                }

                let delta_time = target_peak.1 - anchor_peak_time;
                let hash_key = hash_function(anchor_peak_frequency, target_peak.0, delta_time);
                keys.push(hash_key);
                values.push(anchor_peak_time);
                

                target_count += 1;
                pairs_counter += 1;
                writeln!(writer, "Index: {:?} | Key: {:?}  Value: {:?}", pairs_counter, hash_key, anchor_peak_time)?;

                // [(anchor_peak_frequency, target_peak_frequency, delta_time), anchor_peak_time]
            }


        }
    }
    if !to_recognize {
        insert_fingerprint(conn, keys, values, song_id)?;
        Ok(song_id)
    } else {
        let result: Vec<(u64, u64, f64)> = get_song(conn, &keys)?;
        // let (_, matched_song_id) = match_song(result, &keys, &values, None)?;
        let (confiedence, matched_song_id) = match_song(result, &keys, &values, None)?;
        println!("Confiedence of the matched song: {:?}", confiedence);
        Ok(matched_song_id)
    }
}