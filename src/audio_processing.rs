#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};
use hound::{WavReader, WavWriter, WavSpec, Sample};
use std::fs;
use std::error::Error;
use std::fs::File;
use std::io::{Write, BufWriter};
use crate::constants::OUTPUT_SAMPLE_RATE;

pub fn process_audio(audio_path : &str) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
    // Ensure output directory exists

    // Open the input WAV file
    let full_open_path = format!("../audio/{}", audio_path);
    let mut reader = WavReader::open(full_open_path)?;
    let spec = reader.spec();
    let input_sample_rate = spec.sample_rate;
    let channels = spec.channels as usize;

    println!("Input file: {} channels, {} Hz", channels, input_sample_rate);

    // Read all samples from the file
    let samples: Vec<i16> = reader.samples::<i16>().filter_map(Result::ok).collect();
    
    if samples.is_empty() {
        return Err("Input file contains no samples".into());
    }
    
    println!("Read {} samples from input file", samples.len());
    
    // Organize samples by channel
    let mut waves_in: Vec<Vec<f64>> = vec![Vec::new(); channels];
    
    if channels == 1 {
        // Mono input - just convert samples to f64
        waves_in[0] = samples.iter().map(|&s| s as f64 / 32768.0).collect();
    } else {
        // Multi-channel input - separate channels
        for c in 0..channels {
            waves_in[c] = samples
                .iter()
                .skip(c)
                .step_by(channels)
                .map(|&s| s as f64 / 32768.0)
                .collect();
        }
    }
    
    // Create mono by averaging all channels
    let mono_input: Vec<f64> = if channels == 1 {
        waves_in[0].clone()
    } else {
        let len = waves_in[0].len();
        let mut mono = Vec::with_capacity(len);
        for i in 0..len {
            let sum: f64 = waves_in.iter().map(|channel| channel[i]).sum();
            mono.push(sum / channels as f64);
        }
        mono
    };


    
    println!("Mono input length: {}", mono_input.len());
    // Set up resampler
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    
    // Calculate output length (approximate)
    let output_sample_rate = OUTPUT_SAMPLE_RATE;
    let ratio = output_sample_rate / input_sample_rate as f64;
    let expected_output_len = (mono_input.len() as f64 * ratio).ceil() as usize;
    println!("Expected output length: {} samples", expected_output_len);
    
    // Create resampler with correct ratio (output_rate / input_rate)
    let mut resampler = SincFixedIn::<f64>::new(
        ratio,
        1.0,  // Scaling factor for mono
        params,
        mono_input.len(),  // Use full input as buffer
        1,    // 1 channel (mono)
    )?;
    
    // Process all samples in one go
    let waves_out = resampler.process(&[mono_input], None)?;
    
    if waves_out.is_empty() || waves_out[0].is_empty() {
        return Err("Resampling produced no output".into());
    }


    
    println!("Resampled output length: {}", waves_out[0].len());

    // let file = File::create("./log/audio_sample.txt")?;
    // let mut writer = BufWriter::new(file);
    // for row in &waves_out[0] {
    //     writeln!(writer, "{:?}", row)?; // Writes each row as [1, 2, 3]
    // }
    
    // Create output WAV file
    // let out_spec = WavSpec {
    //     channels: 1,
    //     sample_rate: OUTPUT_SAMPLE_RATE as u32,
    //     bits_per_sample: 16,
    //     sample_format: hound::SampleFormat::Int,
    // };
    // let full_write_path = format!("./processed_audio/{}", audio_path);
    // let mut writer = WavWriter::create(full_write_path, out_spec)?;
    
    // // Write resampled data to output file
    // let mut sample_count = 0;
    // for &sample in &waves_out[0] {
    //     // Scale to i16 range and clamp to prevent overflow
    //     let scaled = (sample * 32767.0).round().max(-32768.0).min(32767.0) as i16;
    //     writer.write_sample(scaled)?;
    //     sample_count += 1;
    // }
    
    // writer.finalize()?;
    // println!("Successfully wrote {} samples to output file", sample_count);
    
    Ok(waves_out)
}