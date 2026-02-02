use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use shazam::fingerprinting::create_hash::build_sample;
use shazam::fingerprinting::make_spectr::FrameAnalyzer;
use shazam::fingerprinting::match_song::{MatchConfig, match_song};
use shazam::fingerprinting::process_spectr::find_spectral_peaks;

fn bench_frame_analyzer(c: &mut Criterion) {
    let audio = generate_audio(8000.0, 10.0);
    let mut analyzer = FrameAnalyzer::new();

    c.bench_function("fingerprinting/frame_analyzer/analyze_peaks", |b| {
        b.iter(|| {
            let peaks = analyzer.analyze_peaks(black_box(&audio), black_box(0));
            black_box(peaks);
        })
    });
}

fn bench_find_spectral_peaks(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprinting/find_spectral_peaks");
    for seconds in [5.0_f32, 20.0_f32] {
        let audio = generate_audio(8000.0, seconds as f64);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{seconds}s")),
            &audio,
            |b, audio| {
                b.iter(|| {
                    let peaks = find_spectral_peaks(black_box(audio)).unwrap();
                    black_box(peaks);
                })
            },
        );
    }
    group.finish();
}

fn bench_build_sample(c: &mut Criterion) {
    let audio = generate_audio(8000.0, 10.0);
    let peaks = find_spectral_peaks(&audio).unwrap();

    c.bench_function("fingerprinting/create_hash/build_sample", |b| {
        b.iter(|| {
            let sample = build_sample(black_box(&peaks));
            black_box(sample);
        })
    });
}

fn bench_match_song(c: &mut Criterion) {
    // Deterministic synthetic workload: lots of match rows for one sample.
    let sample_keys: Vec<u64> = (0..20_000).map(|i| 1_000_000 + i as u64).collect();
    let sample_times: Vec<f32> = (0..20_000).map(|i| i as f32 * 0.01).collect();

    // Create a high-collision scenario: 10 songs share many hashes.
    let mut matches = Vec::with_capacity(sample_keys.len() * 5);
    for (i, &k) in sample_keys.iter().enumerate() {
        let t = sample_times[i];
        for song in 1..=10u64 {
            let song_time = t + (song as f32) * 0.1;
            matches.push((k, song, song_time));
        }
    }

    let config = MatchConfig::default();
    c.bench_function("fingerprinting/match_song", |b| {
        b.iter(|| {
            let result = match_song(
                black_box(&matches),
                black_box(&sample_keys),
                black_box(&sample_times),
                black_box(config),
            )
            .unwrap();
            black_box(result);
        })
    });
}

fn generate_audio(sample_rate: f64, seconds: f64) -> Vec<f64> {
    let len = (sample_rate * seconds) as usize;
    let mut out = Vec::with_capacity(len);

    // Mix two tones and a tiny deterministic pseudo-noise term.
    let w1 = 2.0 * std::f64::consts::PI * 440.0 / sample_rate;
    let w2 = 2.0 * std::f64::consts::PI * 880.0 / sample_rate;
    let mut noise = 0x1234_5678u64;

    for i in 0..len {
        noise ^= noise << 13;
        noise ^= noise >> 7;
        noise ^= noise << 17;
        let n = ((noise & 0xFFFF) as f64 / 65535.0) - 0.5;

        let t = i as f64;
        let s = 0.6 * (w1 * t).sin() + 0.3 * (w2 * t).sin() + 0.01 * n;
        out.push(s);
    }

    out
}

criterion_group!(
    benches,
    bench_frame_analyzer,
    bench_find_spectral_peaks,
    bench_build_sample,
    bench_match_song
);
criterion_main!(benches);
