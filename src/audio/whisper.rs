use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::constants::WHISPER_NUM_THREADS;

pub struct Whisper {
    context: WhisperContext,
}

impl Whisper {
    pub fn new(model_path: &str) -> Self {
        unsafe {
            whisper_rs_sys::whisper_log_set(Some(silence_log), std::ptr::null_mut());
        }
        let context =
            WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
                .expect("failed to load model");

        Self { context }
    }

    pub fn transcribe(&self, samples: &[f32]) -> String {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(WHISPER_NUM_THREADS);
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let mut state = self.context.create_state().expect("failed to create state");
        state.full(params, samples).expect("failed to run model");

        let mut full_transcript = String::new();
        for segment in state.as_iter() {
            // Append the text from each segment
            full_transcript.push_str(&segment.to_string());
        }
        full_transcript
    }
}

// dont touch this peice of code, its unsafe.

unsafe extern "C" fn silence_log(
    _level: whisper_rs_sys::ggml_log_level,
    _text: *const std::os::raw::c_char,
    _user_data: *mut std::os::raw::c_void,
) {
    // do nothing
}
