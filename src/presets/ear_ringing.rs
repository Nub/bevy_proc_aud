use bevy::prelude::*;
use fundsp::prelude32::*;

use crate::dsp::param::ParamHandle;

/// Ear ringing (tinnitus) preset — a cluster of high-frequency sine waves
/// with slight detuning, creating a beating interference pattern.
#[derive(Component, Debug, Clone)]
pub struct EarRinging {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
}

impl Default for EarRinging {
    fn default() -> Self {
        Self { intensity: 0.3 }
    }
}

/// Runtime handles stored alongside the EarRinging entity.
#[derive(Component)]
pub struct EarRingingParams {
    pub intensity: ParamHandle,
}

/// Build the ear ringing DSP graph and return (graph, params).
///
/// Audio-rate sine oscillators for the tinnitus tones (no aliasing),
/// with control-rate LFOs for amplitude modulation (throb + flutter)
/// and stereo rotation to create a disorienting "head spinning" effect.
pub fn build_ear_ringing_graph(er: &EarRinging) -> (Box<dyn AudioUnit>, EarRingingParams) {
    let intensity_param = ParamHandle::new("intensity", er.intensity, 0.0, 1.0);
    let intensity_s = intensity_param.shared().clone();

    // Audio-rate tones: 3 detuned pairs creating beating interference.
    let tones = (dc(4000.0) >> sine()
        + dc(4015.0) >> sine()
        + dc(5200.0) >> sine()
        + dc(5230.0) >> sine()
        + dc(6800.0) >> sine()
        + dc(6790.0) >> sine())
        * dc(1.0 / 6.0)
        * var(&intensity_s);

    // Control-rate amplitude modulation: slow throb + faster flutter.
    let amp_mod = lfo(|t: f32| -> f32 {
        let tau = core::f32::consts::TAU;
        let throb = 0.55 + 0.45 * (tau * 0.14 * t).sin();
        let flutter = 0.8 + 0.2 * (tau * 0.7 * t).sin();
        throb * flutter
    });

    // Mono modulated signal → stereo.
    let stereo = (tones * amp_mod) >> split::<U2>();

    // Per-channel panning LFOs: sound circles the head (~8s cycle).
    let left_gain = lfo(|t: f32| -> f32 {
        let pan = core::f32::consts::TAU * 0.12 * t;
        0.3 + 0.7 * pan.cos().powi(2)
    });
    let right_gain = lfo(|t: f32| -> f32 {
        let pan = core::f32::consts::TAU * 0.12 * t;
        0.3 + 0.7 * pan.sin().powi(2)
    });

    let graph = stereo * (left_gain | right_gain);

    let boxed: Box<dyn AudioUnit> = Box::new(graph);

    let params = EarRingingParams {
        intensity: intensity_param,
    };

    (boxed, params)
}
