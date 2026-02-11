use bevy::prelude::*;
use fundsp::prelude32::*;

use crate::dsp::param::ParamHandle;

/// Heartbeat preset — spawns an ECG-like rhythmic thump.
///
/// Mutate fields at runtime; the sync system pushes changes to the audio thread.
#[derive(Component, Debug, Clone)]
pub struct Heartbeat {
    /// Beats per minute (30–220).
    pub heart_rate: f32,
    /// Random jitter on beat timing (0.0 = perfectly regular, 1.0 = chaotic).
    pub arrhythmic_strength: f32,
    /// Overall intensity (0.0–1.0). Controls volume and low-pass cutoff.
    pub intensity: f32,
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            heart_rate: 72.0,
            arrhythmic_strength: 0.0,
            intensity: 0.5,
        }
    }
}

/// Runtime handles stored alongside the Heartbeat entity.
#[derive(Component)]
pub struct HeartbeatParams {
    pub rate: ParamHandle,
    pub intensity: ParamHandle,
    pub arrhythmia: ParamHandle,
}

/// A damped oscillation burst for a single heart sound.
/// Mixes two harmonics with exponential decay and a short attack ramp.
fn heart_sound(local_t: f32, freq_lo: f32, freq_hi: f32, decay: f32) -> f32 {
    if local_t < 0.0 {
        return 0.0;
    }
    // Short 2ms attack ramp to avoid click, then exponential decay.
    let attack = (local_t * 500.0).min(1.0);
    let env = attack * (-decay * local_t).exp();
    let lo = (core::f32::consts::TAU * freq_lo * local_t).sin();
    let hi = (core::f32::consts::TAU * freq_hi * local_t).sin() * 0.4;
    (lo + hi) * env
}

/// Build the heartbeat DSP graph and return (graph, params).
///
/// Synthesizes a "lub-dub" heartbeat using two damped oscillation bursts:
/// - S1 ("lub"): lower-pitched, longer decay
/// - S2 ("dub"): higher-pitched, shorter decay, ~0.33 beat periods later
pub fn build_heartbeat_graph(hb: &Heartbeat) -> (Box<dyn AudioUnit>, HeartbeatParams) {
    let rate_param = ParamHandle::new("heart_rate", hb.heart_rate, 30.0, 220.0);
    let intensity_param = ParamHandle::new("intensity", hb.intensity, 0.0, 1.0);
    let arrhythmia_param = ParamHandle::new("arrhythmia", hb.arrhythmic_strength, 0.0, 1.0);

    let rate_s = rate_param.shared().clone();
    let intensity_s = intensity_param.shared().clone();
    let arrhythmia_s = arrhythmia_param.shared().clone();

    let graph = lfo(move |t: f32| -> f32 {
        let bpm = rate_s.value().max(30.0);
        let beat_period = 60.0 / bpm;
        let tau = core::f32::consts::TAU;

        // Arrhythmia: multiple incommensurate sine waves create a
        // chaotic-feeling phase jitter. At 0.0 beats are perfectly
        // regular; at 1.0 they're sporadic (~±40% timing variation).
        let arr = arrhythmia_s.value();
        let phase_jitter = arr * 0.4 * (
            (tau * 0.37 * t).sin() * 0.5
            + (tau * 0.83 * t).sin() * 0.3
            + (tau * 1.71 * t).sin() * 0.2
        );
        let phase = (t / beat_period + phase_jitter).fract();

        // S1 ("lub") at phase 0.0 — deep thump.
        let s1_t = phase * beat_period;
        let s1 = heart_sound(s1_t, 45.0, 90.0, 25.0);

        // S2 ("dub") at phase 0.33 — higher, sharper.
        let s2_t = (phase - 0.33) * beat_period;
        let s2 = heart_sound(s2_t, 65.0, 130.0, 35.0) * 0.7;

        (s1 + s2) * intensity_s.value()
    }) >> lowpole_hz(150.0)
       >> split::<U2>();

    let boxed: Box<dyn AudioUnit> = Box::new(graph);

    let params = HeartbeatParams {
        rate: rate_param,
        intensity: intensity_param,
        arrhythmia: arrhythmia_param,
    };

    (boxed, params)
}
