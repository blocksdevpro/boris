use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

use crate::constants::CHANNELS;

pub struct AudioResampler {
    resampler: Option<Fft<f32>>,
    input_rate: u32,
    output_rate: u32,
}

impl AudioResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            resampler: None,
            input_rate,
            output_rate,
        }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let resampler = self.resampler.get_or_insert_with(|| {
            Fft::<f32>::new(
                self.input_rate as usize,
                self.output_rate as usize,
                input.len(), // chunks size
                2,           // sub chunks size
                CHANNELS,
                FixedSync::Input,
            )
            .expect("[ERROR] failed to create resampler!")
        });

        let output_length = resampler.output_frames_max() * resampler.nbr_channels();
        let mut buffer = vec![0.0f32; output_length];

        let input_slice = InterleavedSlice::new(input, CHANNELS, input.len())
            .expect("[ERROR] failed to create input slice for resampling!");
        let mut output_slice = InterleavedSlice::new_mut(&mut buffer, CHANNELS, output_length)
            .expect("[ERROR] failed to create output slice for resampling!");

        resampler
            .process_into_buffer(&input_slice, &mut output_slice, None)
            .expect("[ERROR] resampling failed!");

        buffer
    }
}
