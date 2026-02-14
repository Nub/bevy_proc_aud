use bevy::prelude::*;
use fundsp::prelude32::*;

/// One-shot sword slash — FM synthesis of a metal blade impact.
///
/// Uses FM (frequency modulation) synthesis with high modulation indices
/// to create dense, inharmonic spectra. As the modulation index decays,
/// sidebands vanish from the outside in, creating natural spectral decay
/// (the "shiiing" character). Combined with shaped noise for the initial
/// broadband transient.
///
/// Spawn an entity with this component to trigger the sound.
#[derive(Component, Debug, Clone)]
pub struct SwordSlash {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
}

impl Default for SwordSlash {
    fn default() -> Self {
        Self { intensity: 0.8 }
    }
}

/// Build the sword slash DSP graph. One-shot, no runtime params.
pub fn build_sword_slash_graph(ss: &SwordSlash) -> Box<dyn AudioUnit> {
    let int = ss.intensity;

    // --- FM Voice 1: Low metallic body ---
    // Carrier 720 Hz, modulator 487 Hz (inharmonic ratio ~1.48).
    // Mod index 20 → ~22 sidebands each side → dense, noise-like attack.
    // Mod decay slower than amp decay → stays complex until inaudible.
    let v1 = {
        let fm = (dc(720.0)
            + sine_hz(487.0)
                * lfo(move |t: f32| -> f32 { 20.0 * 487.0 * (-t * 3.0).exp() }))
            >> sine();
        let env = lfo(move |t: f32| -> f32 {
            if t > 1.2 {
                return 0.0;
            }
            (t * 500.0).min(1.0) * (-t * 6.0).exp() * 0.02 * int
        });
        fm * env
    };

    // --- FM Voice 2: Mid presence ---
    // Carrier 2100 Hz, modulator 1430 Hz (ratio ~1.47).
    let v2 = {
        let fm = (dc(2100.0)
            + sine_hz(1430.0)
                * lfo(move |t: f32| -> f32 { 18.0 * 1430.0 * (-t * 5.0).exp() }))
            >> sine();
        let env = lfo(move |t: f32| -> f32 {
            if t > 0.6 {
                return 0.0;
            }
            (t * 500.0).min(1.0) * (-t * 10.0).exp() * 0.015 * int
        });
        fm * env
    };

    // --- FM Voice 3: High shimmer ---
    // Carrier 4200 Hz, modulator 2870 Hz (ratio ~1.46).
    let v3 = {
        let fm = (dc(4200.0)
            + sine_hz(2870.0)
                * lfo(move |t: f32| -> f32 { 12.0 * 2870.0 * (-t * 8.0).exp() }))
            >> sine();
        let env = lfo(move |t: f32| -> f32 {
            if t > 0.3 {
                return 0.0;
            }
            (t * 500.0).min(1.0) * (-t * 15.0).exp() * 0.008 * int
        });
        fm * env
    };

    // --- Noise layer: broadband transient with closing lowpass ---
    // Dynamic cutoff 10kHz → 300Hz creates the "whoosh" quality.
    // (noise | cutoff_lfo) >> lowpole() wires noise to audio input
    // and the LFO to cutoff input of the 2-input lowpole filter.
    let cutoff = lfo(move |t: f32| -> f32 { 300.0 + 9700.0 * (-t * 8.0).exp() });
    let noise_env = lfo(move |t: f32| -> f32 {
        if t > 0.5 {
            return 0.0;
        }
        (t * 1000.0).min(1.0) * (-t * 10.0).exp() * 0.07 * int
    });
    let noise_layer = ((noise() | cutoff) >> lowpole()) * noise_env;

    // --- Mix all layers and split to stereo ---
    let graph = (v1 + v2 + v3 + noise_layer) >> split::<U2>();

    Box::new(graph)
}
