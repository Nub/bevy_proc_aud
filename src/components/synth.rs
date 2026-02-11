use bevy::prelude::*;

/// Marker component that triggers DSP graph construction.
/// Attach `OscillatorType`, `Frequency`, `Amplitude`, and optional filter/effect
/// components to the same entity.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Synth;

/// Oscillator waveform type.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscillatorType {
    Sine,
    Saw,
    Square,
    Triangle,
    Noise,
}

impl Default for OscillatorType {
    fn default() -> Self {
        Self::Sine
    }
}

/// Oscillator frequency in Hz.
#[derive(Component, Debug, Clone, Copy)]
pub struct Frequency(pub f32);

impl Default for Frequency {
    fn default() -> Self {
        Self(440.0)
    }
}

/// Output amplitude (0.0–1.0).
#[derive(Component, Debug, Clone, Copy)]
pub struct Amplitude(pub f32);

impl Default for Amplitude {
    fn default() -> Self {
        Self(0.3)
    }
}
