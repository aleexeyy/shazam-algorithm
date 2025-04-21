use std::collections::HashMap;

struct Song {
    id: u64,
    hashes: Vec<u64>,
    anchor_times: Vec<f64>
}

impl Song {
    fn new(id: u64) -> Self {
        Self {
            id,
            hashes: Vec::new(),
            anchor_times: Vec::new(),
        }
    }
    
    fn new_sample(id: u64, keys: &Vec<u64>, values: &Vec<f64>) -> Self {
        Self {
            id,
            hashes: keys.clone(),
            anchor_times: values.clone(),
        }
    }
    
    fn add(&mut self, hash: u64, anchor_time: f64) {
        self.hashes.push(hash);
        self.anchor_times.push(anchor_time);
    }

    fn create_hashmap(&self) -> HashMap<u64, Vec<f64>> {
        let mut map: HashMap<u64, Vec<f64>> = HashMap::new();
        
        for (hash, anchor_time) in self.hashes.iter().zip(self.anchor_times.iter()) {
            map.entry(*hash).or_insert(Vec::new()).push(*anchor_time);
        }

        map
    }
}

// Configuration struct to make parameters adjustable
pub struct MatchConfig {
    precision_factor: f64,     // Controls time offset precision
    offset_bin_size: i32,      // Size of offset bins for grouping similar offsets
    min_match_threshold: usize, // Minimum matches required to consider a song
    min_confidence: f64,       // Minimum confidence (%) required for a valid match
    max_results: usize,        // Maximum number of results to return
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            precision_factor: 1000.0,
            offset_bin_size: 5,      // Group offsets within 5ms of each other
            min_match_threshold: 2,  // Lowered to 2 for better sensitivity
            min_confidence: 1.0,     // Lowered to 1% to include more potential matches
            max_results: 7,          // Show top 7 results
        }
    }
}

pub fn match_song(
    matches: Vec<(u64, u64, f64)>, 
    keys: &Vec<u64>, 
    values: &Vec<f64>,
    config: Option<MatchConfig>
) -> Result<(f64, u64), Box<dyn std::error::Error>> {
    
    let config = config.unwrap_or_default();

    if matches.is_empty() {
        println!("No songs found to match against.");
        return Ok((0.0, 0));  // Return a default value instead of an error
    }

    let sample = Song::new_sample(0, keys, values);
    let mut songs_hashes: Vec<Song> = Vec::new();
    let mut songs_counter: usize = 0;

    // Group matches by song
    for row in &matches {
        if !songs_hashes.is_empty() && songs_hashes[songs_counter-1].id == row.1 {
            songs_hashes[songs_counter-1].add(row.0, row.2);
        } else {
            let mut song = Song::new(row.1);
            song.add(row.0, row.2);
            songs_hashes.push(song);
            songs_counter += 1;
        }
    }
    
    // Early return if no songs were processed
    if songs_hashes.is_empty() {
        println!("No valid songs to analyze.");
        return Ok((0.0, 0));
    }
    
    // Maps to track both song-offset pairs and their match counts
    let mut offset_counts: HashMap<(u64, i32), usize> = HashMap::new();
    
    // Maps to track hash frequency across songs (for identifying common/ambiguous hashes)
    let mut hash_frequency: HashMap<u64, usize> = HashMap::new();
    
    // First pass: count hash frequency across all songs
    for song in &songs_hashes {
        for &hash in &song.hashes {
            *hash_frequency.entry(hash).or_insert(0) += 1;
        }
    }
    
    // Second pass: perform matching with consideration for hash uniqueness
    for song in &songs_hashes {
        let song_hash_map = song.create_hashmap();
        
        // For each hash in the sample
        for (i, &sample_hash) in sample.hashes.iter().enumerate() {
            let sample_time = sample.anchor_times[i];
            
            // If this hash exists in the song
            if let Some(song_times) = song_hash_map.get(&sample_hash) {
                // Get the frequency of this hash across all songs
                let frequency = hash_frequency.get(&sample_hash).unwrap_or(&1);
                
                // Weight inversely by frequency - more common hashes get lower weight
                let weight = 1.0 / (*frequency as f64).max(1.0);
                
                // Handle collision: Consider all possible time positions for this hash
                for &song_time in song_times {
                    // Calculate the time offset (keeping the sign)
                    let raw_offset = (song_time - sample_time) * config.precision_factor;
                    
                    // Bin the offsets to handle slight timing variations
                    let offset_bin = (raw_offset as i32) / config.offset_bin_size * config.offset_bin_size;
                    
                    let key = (song.id, offset_bin);
                    *offset_counts.entry(key).or_insert(0) += (weight * 100.0) as usize;
                }
            }
        }
    }

    // Reorganize offset counts by song ID for analysis
    let mut song_match_counts: HashMap<u64, HashMap<i32, usize>> = HashMap::new();
    
    for ((song_id, offset), count) in offset_counts {
        song_match_counts
            .entry(song_id)
            .or_insert_with(HashMap::new)
            .insert(offset, count);
    }
    
    // Find the best match for each song
    let mut best_matches: Vec<(u64, i32, usize, f64)> = Vec::new();
    
    for (song_id, offsets) in song_match_counts {
        if let Some((&best_offset, &count)) = offsets.iter().max_by_key(|&(_, count)| count) {
            // Skip if below minimum match threshold
            if count < config.min_match_threshold {
                continue;
            }
            
            // Find the matching song to calculate confidence
            if let Some(matched_song) = songs_hashes.iter().find(|s| s.id == song_id) {
                let confidence = (count as f64 / matched_song.hashes.len() as f64) * 100.0;
                
                // Skip if below minimum confidence threshold
                if confidence < config.min_confidence {
                    continue;
                }
                
                best_matches.push((song_id, best_offset, count, confidence));
            }
        }
    }
    
    // Sort matches by confidence (highest first)
    best_matches.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    
    // Handle empty results
    if best_matches.is_empty() {
        println!("No matches met the minimum criteria (threshold: {} matches, {:.1}% confidence)",
                config.min_match_threshold, config.min_confidence);
        return Ok((0.0, 0));  // Return default value for no matches
    }
    
    // Display top matches
    let mut top_confidence: f64 = 0.0;
    let mut top_song_id: u64 = 0;
    
    println!("Top matches:");
    for (i, (song_id, offset, count, confidence)) in best_matches.iter().take(config.max_results).enumerate() {
        let offset_seconds = *offset as f64 / config.precision_factor;
        
        // Whether the sample is earlier or later than the matched song
        let position = if offset_seconds > 0.0 {
            "sample behind song"
        } else if offset_seconds < 0.0 {
            "sample ahead of song"
        } else {
            "exact match"
        };
        
        println!("{}. Song ID {}: {} weighted matches, {:.2}% confidence at offset {:.3} seconds ({})", 
                 i+1, song_id, count, confidence, offset_seconds.abs(), position);
        
        if *confidence > top_confidence {
            top_confidence = *confidence;
            top_song_id = *song_id;
        }
    }
    
    println!("Number of songs analyzed: {}", songs_counter);
    println!("Total number of fingerprint matches: {}", matches.len());
    
    Ok((top_confidence, top_song_id))
}