use crate::fingerprinting::constants::OUTPUT_SAMPLE_RATE;
use crate::fingerprinting::error::FingerprintError;
use crate::paths;
use hound::WavReader;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

pub fn process_audio(audio_path: &str) -> Result<Vec<f64>, FingerprintError> {
    let full_open_path = paths::audio_dir().join(audio_path);
    let mut reader = WavReader::open(full_open_path)?;
    let spec = reader.spec();
    let input_sample_rate = spec.sample_rate;
    let channels = usize::from(spec.channels);

    if channels == 0 {
        return Err(FingerprintError::InvalidAudio("wav has 0 channels"));
    }

    let samples: Vec<i16> = reader.samples::<i16>().collect::<Result<Vec<_>, _>>()?;
    if samples.is_empty() {
        return Err(FingerprintError::EmptyAudio);
    }

    let mut mono_input = Vec::with_capacity(samples.len() / channels);
    if channels == 1 {
        mono_input.extend(samples.iter().map(|&s| s as f64 / 32768.0));
    } else {
        for frame in samples.chunks_exact(channels) {
            let sum: i32 = frame.iter().map(|&s| s as i32).sum();
            mono_input.push(sum as f64 / (channels as f64 * 32768.0));
        }
    }

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let ratio = OUTPUT_SAMPLE_RATE / input_sample_rate as f64;
    let mut resampler = SincFixedIn::<f64>::new(ratio, 1.0, params, mono_input.len(), 1)?;
    let mut waves_out = resampler.process(&[mono_input], None)?;

    match waves_out.pop() {
        Some(out) if !out.is_empty() => Ok(out),
        _ => Err(FingerprintError::InvalidAudio(
            "resampling produced no output",
        )),
    }
}
