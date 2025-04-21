#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]
use rustfft::{FftPlanner, num_complex::Complex};
use std::f64::consts::PI;
use std::cmp::min;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::f64::consts::LOG10_E;
use crate::constants::*;


pub fn window_function(frame : &[f64]) -> Vec<f64> {
    let frame_length = frame.len();
    let mut windowed_frame : Vec<f64> = vec![0.0; frame_length];
    for n in 0..frame_length {
        windowed_frame[n] = frame[n] * 0.5 * (1.0 - (2.0 * PI * n as f64 / (frame_length as f64 - 1.0)).cos());
    }
    return windowed_frame;
}

pub fn compute_fft(frame: &[f64]) -> Vec<f64> {
    // Create FFT planner
    let mut planner = FftPlanner::new();
    
    // Create a real-to-complex FFT (transform real data into complex)
    let fft = planner.plan_fft_forward(frame.len());

    // Prepare input data as complex numbers (real part, imaginary part)
    let mut input: Vec<Complex<f64>> = frame.iter().map(|&x| Complex::new(x, 0.0)).collect();

    // Perform FFT
    fft.process(&mut input);

    // Extract magnitudes (complex numbers have a real and imaginary part)
    
    input.iter().map(|c| {
        let magnitude = c.norm();
        
        // Print a few values to debug (remove in production)
        // if magnitude == 0.0 {
        //     println!("Very small magnitude: {}", magnitude);
        // }
        
        // Set an explicit threshold for "zero" values
        if magnitude < 1e-10 {
            -200.0  // Fixed floor value
        } else {
            20.0 * magnitude.log10()
        }
    }).collect()
    // input.iter().map(|c| c.norm()).collect()
}

pub fn window_audio(audio : Vec<Vec<f64>>) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {

    // let audio: Vec<Vec<f64>> = audio_processing::process_audio()?;
    let audio: Vec<f64> = audio[0].clone();

    

    let frame_length = FRAME_LENGTH;
    let hop_length = HOP_LENGTH;
    let total_samples = audio.len();
    let remainder = total_samples % hop_length;
    let padding_needed = if remainder == 0 {0} else {hop_length-remainder};


    let mut padded_audio = audio.to_vec();
    padded_audio.extend(vec![0.0; padding_needed]);
    println!("Audio size: {} samples ({:.2} seconds)", 
             padded_audio.len(), 
             padded_audio.len() as f64 / OUTPUT_SAMPLE_RATE);
    let mut spectrogram: Vec<Vec<f64>> = Vec::with_capacity((padded_audio.len() - frame_length) / hop_length + 1);
    

    for i in (0..padded_audio.len()-frame_length+1).step_by(hop_length) {
        let frame = window_function(&padded_audio[i..(i + frame_length)]);



        let fft_magnitude = compute_fft(&frame);

        spectrogram.push(fft_magnitude);
    }
    println!("Generated spectrogram with {} time frames.", spectrogram.len());


    let file = File::create("../log/log_spectrogram.txt")?;
    let mut writer = BufWriter::new(file);
    for row in &spectrogram {
        writeln!(writer, "{:?}", row)?; // Writes each row as [1, 2, 3]
    }

    // visualize_spectrogram(&spectrogram)?;
    
    Ok(spectrogram)
}


// pub fn visualize_spectrogram(spectrogram: &Vec<Vec<f64>>) -> Result<(), Box<dyn Error>> {
//     use bmp::{Image, Pixel};
//     use std::path::Path;

//     if spectrogram.is_empty() || spectrogram[0].is_empty() {
//         return Err("Empty spectrogram data".into());
//     }

//     // Determine dimensions safely
//     let height = spectrogram[0].len() / 2;
//     let width = spectrogram.len();

//     if height == 0 || width == 0 {
//         return Err("Invalid spectrogram dimensions".into());
//     }

//     println!("Creating spectrogram image: {}x{} pixels", width, height);

//     // Create a new image
//     let mut img = Image::new(width as u32, height as u32);

//     // Find the maximum value for normalization
//     let mut max_value: f64 = 1e-10; // Avoid division by zero
//     for frame in spectrogram {
//         for &value in frame.iter().take(height) {
//             if value > max_value {
//                 max_value = value;
//             }
//         }
//     }

//     // Apply logarithmic scaling and normalization
//     for (x, frame) in spectrogram.iter().enumerate() {
//         for y in 0..height {
//             if y < frame.len() {
//                 // Apply log scaling for better visualization (dB scale)
//                 let db_value = 20.0 * (frame[y] / max_value).max(1e-10).log10();

//                 // Normalize to 0-255 range (typical range is -80dB to 0dB)
//                 let normalized = ((db_value + 80.0) / 80.0).clamp(0.0, 1.0);

//                 // Apply color mapping (blue to white colormap)
//                 let intensity = (normalized * 255.0) as u8;

//                 // **FIXED**: Ensure `height - 1 - y` does not underflow
//                 let y_pos = height.saturating_sub(1).saturating_sub(y);
                
//                 let intensity_i16 = intensity as i16;
//                 // Set pixel color (heatmap from blue to white)
//                 let pixel = if normalized < 0.25 {
//                     // Dark blue to blue
//                     Pixel::new(0, 0, ((intensity_i16 * 4).clamp(0, 255)) as u8)
//                 } else if normalized < 0.5 {
//                     // Blue to cyan
//                     Pixel::new(0, ((intensity_i16 - 64) * 4).clamp(0, 255) as u8, 255)
//                 } else if normalized < 0.75 {
//                     // Cyan to yellow
//                     Pixel::new(
//                         ((intensity_i16 - 128) * 4).clamp(0, 255) as u8,
//                         255,
//                         (255 - (intensity_i16 - 128) * 4).clamp(0, 255) as u8,
//                     )
//                 } else {
//                     // Yellow to white
//                     Pixel::new(255, 255, ((intensity_i16 - 192) * 4).clamp(0, 255) as u8)
//                 };

//                 img.set_pixel(x as u32, y_pos as u32, pixel);
//             }
//         }
//     }

//     // Save the image
//     img.save("spectrogram.bmp")?;
//     println!("Spectrogram saved as 'spectrogram.bmp'");

//     Ok(())
// }
