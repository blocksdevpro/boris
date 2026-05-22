use std::sync::mpsc;
use std::time::Instant;

use crate::Event;
use crate::audio::buffer::{RecordBuffer, SlidingBuffer};
use crate::audio::resampler::AudioResampler;
use crate::audio::stream::AudioStream;
use crate::constants::{SAMPLE_RATE, WAKEWORD_INTERVAL};

#[derive(PartialEq)]
enum RecordingState {
    Idle,
    Capturing,
    Transcribing,
}

pub struct AudioAdapter {
    stream: AudioStream,
    resampler: AudioResampler,
    wakeword_buffer: SlidingBuffer,
    transcribe_buffer: RecordBuffer,
    recording_state: RecordingState,
}

impl AudioAdapter {
    pub fn from_stream(stream: AudioStream) -> Self {
        let input_rate = stream.rate();
        let resampler = AudioResampler::new(input_rate, SAMPLE_RATE);
        Self {
            stream,
            resampler,
            wakeword_buffer: SlidingBuffer::new(SAMPLE_RATE as usize * 2),
            transcribe_buffer: RecordBuffer::new(SAMPLE_RATE as usize * 100),
            recording_state: RecordingState::Idle,
        }
    }

    pub fn process(&mut self, tx: mpsc::Sender<Event>) {
        self.stream.play();
        let mut timestamp = Instant::now();

        loop {
            let frame = self.stream.read();
            let processed = self.resampler.process(&frame);

            self.append(&processed);
            if self.wakeword_buffer.ready()
                && self.recording_state == RecordingState::Idle
                && timestamp.elapsed() >= WAKEWORD_INTERVAL
            {
                let samples = self.read();
                tx.send(Event::Process(samples)).ok();
                timestamp = Instant::now();
            }
        }
    }

    pub fn state_capturing(&mut self) {
        self.transcribe_buffer.trim(SAMPLE_RATE as usize);
        self.recording_state = RecordingState::Capturing;
    }

    pub fn state_transcribing(&mut self) {
        self.recording_state = RecordingState::Transcribing;
        self.transcribe_buffer.clear();
    }

    pub fn state_idle(&mut self) {
        self.recording_state = RecordingState::Idle;
        self.transcribe_buffer.clear();
    }

    pub fn append(&mut self, data: &[f32]) {
        // extend the wakeword buffer and transcribe buffer if recording is active
        self.wakeword_buffer.push(data);

        if self.recording_state == RecordingState::Idle
            || self.recording_state == RecordingState::Capturing
        {
            self.transcribe_buffer.push(data);
        }
    }

    pub fn read(&self) -> Vec<f32> {
        self.wakeword_buffer.read()
    }
}
