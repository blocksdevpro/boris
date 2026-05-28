// File: src/audio/filters.rs

/// Removes low-frequency vehicle rumble (engine/road noise) below 150Hz.
pub fn high_pass_filter(samples: &[f32]) -> Vec<f32> {
    let alpha = 0.94f32;
    let mut filtered = Vec::with_capacity(samples.len());
    let mut prev_input = 0.0f32;
    let mut prev_output = 0.0f32;

    for &x in samples {
        let y = alpha * (prev_output + x - prev_input);
        prev_input = x;
        prev_output = y;
        filtered.push(y);
    }
    filtered
}

/// Boosts the quiet high-frequency consonants to make the wake word crisper.
pub fn pre_emphasis(samples: &[f32]) -> Vec<f32> {
    let alpha = 0.97f32;
    let mut output = Vec::with_capacity(samples.len());
    if !samples.is_empty() {
        output.push(samples[0]);
    }
    for i in 1..samples.len() {
        let y = samples[i] - alpha * samples[i - 1];
        output.push(y);
    }
    output
}

/// Normalizes the average volume (RMS) to a target level.
/// This prevents sudden loud noise spikes from muting the voice.
pub fn rms_normalize(samples: &mut [f32], target_rms: f32) {
    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    let rms = (sum_squares / samples.len() as f32).sqrt();

    if rms > 0.005 {
        let gain = target_rms / rms;
        let gain = gain.min(5.0); // limit maximum boost to avoid blowing up static noise
        for sample in samples.iter_mut() {
            *sample *= gain;
        }
    }
}
