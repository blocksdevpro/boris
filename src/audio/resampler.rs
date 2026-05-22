use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

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
                input.len(),
                2,
                1,
                FixedSync::Input,
            )
            .unwrap()
        });

        let output_length = resampler.output_frames_max() * resampler.nbr_channels();
        let mut buffer = vec![0.0f32; output_length];

        let input_slice = InterleavedSlice::new(input, 1, input.len()).unwrap();
        let mut output_slice = InterleavedSlice::new_mut(&mut buffer, 1, output_length).unwrap();

        resampler
            .process_into_buffer(&input_slice, &mut output_slice, None)
            .expect("[Error] resampling failed!");

        buffer
    }
}
