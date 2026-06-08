use hound::{SampleFormat, WavSpec, WavWriter};

pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|s| (*s * i16::MAX as f32) as i16)
        .collect()
}

#[allow(dead_code)]
pub fn write_wav(filename: &str, samples: &[i16], sample_rate: u32) {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer =
        WavWriter::create(filename, spec).expect("[ERROR] failed to create WAV writer!");
    for &sample in samples {
        writer
            .write_sample(sample)
            .expect("[ERROR] failed to write sample to WAV writer!");
    }
}
