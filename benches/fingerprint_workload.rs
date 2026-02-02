use shazam::fingerprinting::create_hash::build_sample;
use shazam::fingerprinting::process_spectr::find_spectral_peaks;

fn main() {
    let seconds: f64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20.0);
    let iterations: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let audio = generate_audio(8000.0, seconds);

    for _ in 0..iterations {
        let peaks = find_spectral_peaks(&audio).expect("peak extraction failed");
        let sample = build_sample(&peaks);
        std::hint::black_box(sample);
    }
}

fn generate_audio(sample_rate: f64, seconds: f64) -> Vec<f64> {
    let len = (sample_rate * seconds) as usize;
    let mut out = Vec::with_capacity(len);

    let w1 = 2.0 * std::f64::consts::PI * 440.0 / sample_rate;
    let w2 = 2.0 * std::f64::consts::PI * 660.0 / sample_rate;
    let w3 = 2.0 * std::f64::consts::PI * 880.0 / sample_rate;
    let mut noise = 0xDEAD_BEEF_CAFE_BABEu64;

    for i in 0..len {
        noise ^= noise << 13;
        noise ^= noise >> 7;
        noise ^= noise << 17;
        let n = ((noise & 0xFFFF) as f64 / 65535.0) - 0.5;

        let t = i as f64;
        let s = 0.5 * (w1 * t).sin() + 0.3 * (w2 * t).sin() + 0.2 * (w3 * t).sin() + 0.01 * n;
        out.push(s);
    }

    out
}
