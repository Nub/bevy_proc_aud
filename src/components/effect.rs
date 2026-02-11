use bevy::prelude::*;

/// Reverb effect. Attach to a `Synth` entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Reverb {
    pub room_size: f32,
    pub decay_time: f32,
    pub damping: f32,
    /// Wet/dry mix (0.0 = fully dry, 1.0 = fully wet).
    pub mix: f32,
}

impl Default for Reverb {
    fn default() -> Self {
        Self {
            room_size: 0.5,
            decay_time: 1.5,
            damping: 0.3,
            mix: 0.3,
        }
    }
}

/// Delay effect. Attach to a `Synth` entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Delay {
    pub time_seconds: f32,
    pub feedback: f32,
    /// Wet/dry mix (0.0 = fully dry, 1.0 = fully wet).
    pub mix: f32,
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            time_seconds: 0.3,
            feedback: 0.4,
            mix: 0.3,
        }
    }
}

/// Distortion effect (soft-clip waveshaper). Attach to a `Synth` entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Distortion {
    /// Drive amount (1.0 = clean, higher = more distortion).
    pub drive: f32,
    /// Wet/dry mix (0.0 = fully dry, 1.0 = fully wet).
    pub mix: f32,
}

impl Default for Distortion {
    fn default() -> Self {
        Self {
            drive: 2.0,
            mix: 0.5,
        }
    }
}
