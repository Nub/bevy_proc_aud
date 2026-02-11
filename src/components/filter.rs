use bevy::prelude::*;

/// Low-pass filter. Attach to a `Synth` entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct LowPass {
    pub cutoff_hz: f32,
    pub resonance: f32,
}

impl Default for LowPass {
    fn default() -> Self {
        Self {
            cutoff_hz: 1000.0,
            resonance: 1.0,
        }
    }
}

/// High-pass filter. Attach to a `Synth` entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct HighPass {
    pub cutoff_hz: f32,
    pub resonance: f32,
}

impl Default for HighPass {
    fn default() -> Self {
        Self {
            cutoff_hz: 200.0,
            resonance: 1.0,
        }
    }
}

/// Band-pass filter. Attach to a `Synth` entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct BandPass {
    pub center_hz: f32,
    pub bandwidth: f32,
}

impl Default for BandPass {
    fn default() -> Self {
        Self {
            center_hz: 1000.0,
            bandwidth: 200.0,
        }
    }
}
