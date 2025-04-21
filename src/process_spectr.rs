#![allow(dead_code)]
use ndarray::Array2;
use std::fs::File;
use std::io::{Write, BufWriter};

pub fn convert_to_ndarray(spectrogram: &Vec<Vec<f64>>) -> Array2<f64> {
    let rows = spectrogram.len();
    let cols = spectrogram[0].len();
    let flat_data: Vec<f64> = spectrogram.iter().flatten().copied().collect();
    Array2::from_shape_vec((rows, cols), flat_data).expect("Failed to create ndarray")
}


pub fn find_spectral_peaks(spectrogram: &Vec<Vec<f64>>) -> Result<Vec<Vec<usize>>, Box<dyn std::error::Error>> {

    let spectrogram = &convert_to_ndarray(spectrogram);
    let mut peaks: Vec<Vec<usize>> = vec![Vec::new(); spectrogram.nrows()];
    
    // Frequency bands (bin indices)
    let bands = [
        (4, 6),    // 30-40 Hz
        (6, 11),   // 40-80 Hz
        (11, 16),  // 80-120 Hz
        (16, 24),  // 120-180 Hz
        (24, 40),  // 180-300 Hz
        (40, 65),  // 300-500 Hz
    ];
    
    for i in 0..spectrogram.nrows() {
        for &(start, end) in &bands {
            // Find maximum magnitude in this frequency band
            let mut max_val = f64::MIN;
            let mut max_idx = start;
            
            for j in start..end {
                if j < spectrogram.ncols() && spectrogram[[i, j]] > max_val {
                    max_val = spectrogram[[i, j]];
                    max_idx = j;
                }
            }
            
            // Add the peak frequency bin index to our result
            peaks[i].push(max_idx);
        }
    }

    
    
    let proccessed_peaks:Vec<Vec<usize>>  = filter_peaks_by_threshold(&mut peaks, spectrogram);
    // for i in 0..proccessed_peaks.len() {
    //     println!("The peaks at row {:?}: ", i);
    //     for j in 0..proccessed_peaks[i].len() {
    //         println!("Peak number {:?}: {:?}", j, spectrogram[[i, proccessed_peaks[i][j]]]);
    //     }
    // }
    let file = File::create("../log/filtered_peaks_testing.txt")?;
    let mut writer = BufWriter::new(file);
    for i in 0..proccessed_peaks.len() {
        writeln!(writer, "The peaks at row {:?}: ", i)?;
        for j in 0..proccessed_peaks[i].len() {
            writeln!(writer, "Peak number {:?}: {:?}", j, spectrogram[[i, proccessed_peaks[i][j]]])?;
        }
    }
    Ok(proccessed_peaks.clone())
}

pub fn filter_peaks_by_threshold(peaks: &mut Vec<Vec<usize>>, spectrogram: &Array2<f64>) -> Vec<Vec<usize>> {
    let mut filtered_peaks = vec![Vec::new(); peaks.len()];
    
    // For each time frame
    for i in 0..peaks.len() {
        if i >= spectrogram.nrows() || peaks[i].len() < 6 {
            continue; // Skip if row doesn't exist or doesn't have enough peaks
        }
        
        // Get magnitudes of the 6 frequency bands for this time frame
        let mut band_magnitudes: Vec<f64> = Vec::with_capacity(6);
        for &freq_idx in &peaks[i] {
            if freq_idx < spectrogram.ncols() {
                band_magnitudes.push(spectrogram[[i, freq_idx]]);
            }
        }
        
        // Calculate average of the 6 band magnitudes (if we have all 6)
        if band_magnitudes.len() == 6 {
            let threshold: f64 = band_magnitudes.iter().sum::<f64>() / 6.0;
            
            // Keep only peaks that exceed the threshold
            for (_j, &freq_idx) in peaks[i].iter().enumerate() {
                if freq_idx < spectrogram.ncols() && spectrogram[[i, freq_idx]] > threshold {
                    filtered_peaks[i].push(freq_idx);
                }
            }
        }
    }
    
    return filtered_peaks;
}