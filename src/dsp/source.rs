use bevy::audio::Source;
use bevy::prelude::*;
use fundsp::audiounit::AudioUnit;
use fundsp::buffer::{BufferRef, BufferVec};
use fundsp::MAX_BUFFER_SIZE;
use std::sync::{Arc, Mutex};

/// The `Asset` type bridging FunDSP audio graphs to Bevy's audio pipeline.
///
/// Contains a FunDSP `AudioUnit` that generates samples on the audio thread.
/// The `Mutex` is only contested once at decoder creation — the decoder then
/// owns the unit exclusively.
#[derive(Asset, TypePath)]
pub struct ProceduralAudio {
    graph: Arc<Mutex<Box<dyn AudioUnit>>>,
    sample_rate: u32,
    channels: u16,
}

impl ProceduralAudio {
    pub fn new(mut graph: Box<dyn AudioUnit>, sample_rate: u32, channels: u16) -> Self {
        graph.set_sample_rate(sample_rate as f64);
        graph.allocate();
        Self {
            graph: Arc::new(Mutex::new(graph)),
            sample_rate,
            channels,
        }
    }
}

/// Iterator that pulls samples from a FunDSP graph for rodio playback.
pub struct ProceduralAudioDecoder {
    graph: Box<dyn AudioUnit>,
    sample_rate: u32,
    channels: u16,
    /// FunDSP output buffer for block processing.
    output_buf: BufferVec,
    /// Interleaved sample buffer for rodio.
    buffer: Vec<f32>,
    pos: usize,
    total: usize,
}

impl ProceduralAudioDecoder {
    fn fill_block(&mut self) {
        let ch = self.channels as usize;
        let size = MAX_BUFFER_SIZE;
        self.buffer.resize(size * ch, 0.0);

        let input = BufferRef::empty();
        let mut output = self.output_buf.buffer_mut();
        self.graph.process(size, &input, &mut output);

        // Interleave channels into the flat buffer.
        for i in 0..size {
            let base = i * ch;
            self.buffer[base] = output.at_f32(0, i);
            if ch >= 2 {
                self.buffer[base + 1] = output.at_f32(1, i);
            }
        }

        self.total = size * ch;
        self.pos = 0;
    }
}

impl Iterator for ProceduralAudioDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.total {
            self.fill_block();
        }

        let sample = self.buffer[self.pos];
        self.pos += 1;
        Some(sample)
    }
}

impl Source for ProceduralAudioDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl bevy::audio::Decodable for ProceduralAudio {
    type DecoderItem = f32;
    type Decoder = ProceduralAudioDecoder;

    fn decoder(&self) -> Self::Decoder {
        let graph = self.graph.lock().expect("ProceduralAudio graph lock poisoned");
        let mut cloned = graph.clone();
        cloned.set_sample_rate(self.sample_rate as f64);
        cloned.allocate();
        let ch = self.channels as usize;
        ProceduralAudioDecoder {
            graph: cloned,
            sample_rate: self.sample_rate,
            channels: self.channels,
            output_buf: BufferVec::new(ch),
            buffer: vec![0.0; MAX_BUFFER_SIZE * ch],
            pos: MAX_BUFFER_SIZE * ch, // force fill on first call
            total: MAX_BUFFER_SIZE * ch,
        }
    }
}
