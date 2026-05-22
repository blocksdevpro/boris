use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::Event;
use crate::audio::resampler::AudioResampler;
use crate::audio::stream::AudioStream;
use crate::constants::{SAMPLE_RATE, WAKEWORD_INTERVAL};

#[derive(PartialEq)]
enum RecordingState {
    Idle,
    Capturing,
    Transcribing,
}

struct SlidingBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
}

impl SlidingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    pub fn push(&mut self, samples: &[f32]) {
        for &s in samples {
            if self.buffer.len() == self.capacity {
                self.buffer.pop_front(); // O(1), no shifting
            }
            self.buffer.push_back(s); // O(1) amortized
        }
    }

    pub fn ready(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    pub fn read(&self) -> Vec<f32> {
        let size = self.capacity.min(self.buffer.len());
        let start = self.buffer.len() - size;
        self.buffer.range(start..).copied().collect()
    }
}
pub struct AudioAdapter {
    stream: AudioStream,
    resampler: AudioResampler,
    wakeword_buffer: SlidingBuffer,
    transcribe_buffer: VecDeque<f32>,
    recording_state: RecordingState,
}

impl AudioAdapter {
    pub fn from_stream(stream: AudioStream) -> Self {
        let input_rate = stream.rate();
        let resampler = AudioResampler::new(input_rate, SAMPLE_RATE);
        Self {
            stream,
            resampler,
            wakeword_buffer: SlidingBuffer::new((SAMPLE_RATE * 2) as usize),
            transcribe_buffer: VecDeque::new(),
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
        self.transcribe_buffer.clear();
        self.recording_state = RecordingState::Capturing;
    }

    pub fn state_transcribing(&mut self) {
        self.recording_state = RecordingState::Transcribing;
    }

    pub fn state_idle(&mut self) {
        self.recording_state = RecordingState::Idle;
    }

    pub fn append(&mut self, data: &[f32]) {
        // extend the wakeword buffer and transcribe buffer if recording is active
        self.wakeword_buffer.push(data);

        // extend the transcribe buffer if recording is active
        if self.recording_state == RecordingState::Capturing {
            self.transcribe_buffer.extend(data);
        }
    }

    pub fn read(&self) -> Vec<f32> {
        self.wakeword_buffer.read()
    }
}
