use std::sync::mpsc;
use std::time::Instant;

use crate::audio::boris::BorisEvent;
use crate::audio::buffer::{RecordBuffer, SlidingBuffer};
use crate::audio::resampler::AudioResampler;
use crate::audio::stream::AudioStream;
use crate::constants::{SAMPLE_RATE, VAD_INTERVAL, VAD_SAMPLE_LEN, WAKEWORD_INTERVAL};

pub enum AdapterCommand {
    StartCapture,
    StopCapture,
    Reset,
}

pub struct AudioAdapter {
    stream: AudioStream,
    resampler: AudioResampler,
    command_rx: mpsc::Receiver<AdapterCommand>,
    wakeword_buffer: SlidingBuffer,
    transcribe_buffer: RecordBuffer,
}

impl AudioAdapter {
    pub fn from_stream(stream: AudioStream, command_rx: mpsc::Receiver<AdapterCommand>) -> Self {
        let input_rate = stream.rate();
        let resampler = AudioResampler::new(input_rate, SAMPLE_RATE);

        Self {
            stream,
            resampler,
            command_rx,
            wakeword_buffer: SlidingBuffer::new(SAMPLE_RATE as usize * 2),
            transcribe_buffer: RecordBuffer::new(SAMPLE_RATE as usize * 100),
        }
    }

    pub fn process(&mut self, event_tx: mpsc::Sender<BorisEvent>) {
        self.stream.play();
        let mut wakeword_timestamp = Instant::now();
        let mut vad_timestamp = Instant::now();

        let mut capturing = false;

        loop {
            while let Ok(command) = self.command_rx.try_recv() {
                match command {
                    AdapterCommand::StartCapture => {
                        capturing = true;
                    }
                    AdapterCommand::StopCapture => {
                        capturing = false;
                        event_tx
                            .send(BorisEvent::ProcessTranscribe(self.transcribe_buffer.read()))
                            .ok();
                    }
                    AdapterCommand::Reset => {
                        self.transcribe_buffer.clear();
                        wakeword_timestamp = Instant::now();
                        vad_timestamp = Instant::now();
                    }
                }
            }
            let frame = self.stream.read();
            let processed = self.resampler.process(&frame);

            self.wakeword_buffer.push(&processed);
            if capturing {
                self.transcribe_buffer.push(&processed);
            }

            if self.wakeword_buffer.ready()
                && !capturing
                && wakeword_timestamp.elapsed() >= WAKEWORD_INTERVAL
            {
                let samples = self.wakeword_buffer.read();
                event_tx.send(BorisEvent::ProcessWakeword(samples)).ok();
                wakeword_timestamp = Instant::now();
            }
            if capturing && vad_timestamp.elapsed() >= VAD_INTERVAL {
                // take first VAD_SAMPLE_LEN: 256 samples
                let samples = self.wakeword_buffer.read_last(VAD_SAMPLE_LEN);

                event_tx.send(BorisEvent::ProcessVAD(samples)).ok();
                vad_timestamp = Instant::now();
            }
        }
    }

    pub fn read(&self) -> Vec<f32> {
        self.wakeword_buffer.read()
    }
}
