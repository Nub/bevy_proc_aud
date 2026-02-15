use bevy::prelude::*;
use fundsp::prelude32::*;

/// One-shot explosion / fireball sound effect.
///
/// Five layers: initial broadband blast, sub-bass boom, mid-frequency body,
/// pitch-swept "fireball whoosh", and high-frequency crackle tail.
/// At low pitch_shift values it sounds like a deep detonation; at higher
/// values the whoosh layer dominates for a fiery, swooshing fireball.
/// Duration ~2s.
///
/// Spawn an entity with this component to trigger the sound.
#[derive(Component, Debug, Clone)]
pub struct Explosion {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
    /// Pitch multiplier (1.0 = standard explosion, >1 = fiery fireball, <1 = deep boom).
    pub pitch_shift: f32,
    /// Reverb wet/dry mix (0.0 = dry, 1.0 = fully wet).
    pub reverb_mix: f32,
    /// Low-pass filter cutoff in Hz applied to the whole output (20_000 = effectively off).
    pub lowpass: f32,
}

impl Default for Explosion {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            pitch_shift: 1.0,
            reverb_mix: 0.1,
            lowpass: 20_000.0,
        }
    }
}

/// Build the explosion DSP graph. One-shot, no runtime params.
pub fn build_explosion_graph(ex: &Explosion) -> Box<dyn AudioUnit> {
    let int = ex.intensity;
    let pitch = ex.pitch_shift;
    let reverb_mix = ex.reverb_mix;
    let lowpass = ex.lowpass;

    // Decay speed scales with pitch: higher pitch = faster decay (small fireball),
    // lower pitch = slower decay (massive explosion).
    let decay_scale = pitch.sqrt();

    // --- Layer 1: Initial blast (broadband transient) ---
    // Lowpassed noise burst — pitch controls how bright the crack is.
    let blast_env = lfo(move |t: f32| -> f32 {
        if t > 0.2 / decay_scale {
            return 0.0;
        }
        let attack = (t * 5000.0).min(1.0);
        let decay = (-t * 18.0 * decay_scale).exp();
        attack * decay * 0.2 * int
    });
    let blast_layer = (noise() >> lowpole_hz(3000.0 * pitch)) * blast_env;

    // --- Layer 2: Tonal boom (pitched sine thump) ---
    // Low sine tone that shifts with pitch — subtle pitch cue under the noise.
    let boom_freq = 80.0 * pitch;
    let boom_harm = 130.0 * pitch;
    let boom_env = lfo(move |t: f32| -> f32 {
        if t > 2.5 / decay_scale {
            return 0.0;
        }
        let attack = (t * 60.0).min(1.0);
        let decay = (-t * 1.5 * decay_scale).exp();
        attack * decay * 0.12 * int
    });
    let boom_layer = (sine_hz(boom_freq) + sine_hz(boom_harm) * dc(0.5)) * boom_env;

    // --- Layer 3: Sub rumble (noise-based low end) ---
    let rumble_cutoff = 250.0 * pitch;
    let rumble_env = lfo(move |t: f32| -> f32 {
        if t > 3.0 / decay_scale {
            return 0.0;
        }
        let attack = (t * 80.0).min(1.0);
        let decay = (-t * 1.0 * decay_scale).exp();
        attack * decay * 0.6 * int
    });
    let rumble_layer =
        (noise() >> lowpole_hz(rumble_cutoff) >> lowpole_hz(rumble_cutoff)) * rumble_env;

    // --- Layer 4: Mid body ---
    let mid_cutoff = 800.0 * pitch;
    let mid_env = lfo(move |t: f32| -> f32 {
        if t > 1.5 / decay_scale {
            return 0.0;
        }
        let attack = (t * 150.0).min(1.0);
        let decay = (-t * 2.5 * decay_scale).exp();
        attack * decay * 0.4 * int
    });
    let mid_layer = (noise() >> lowpole_hz(mid_cutoff)) * mid_env;

    // --- Layer 5: Fireball whoosh (pitch-swept noise) ---
    let whoosh_hi = 4000.0 * pitch;
    let whoosh_lo = 200.0 * pitch;
    let whoosh_src = noise();
    let whoosh_cutoff = lfo(move |t: f32| -> f32 {
        whoosh_lo + (whoosh_hi - whoosh_lo) * (-t * 3.0 * decay_scale).exp()
    });
    let whoosh_env = lfo(move |t: f32| -> f32 {
        if t > 1.5 / decay_scale {
            return 0.0;
        }
        let onset = ((t - 0.02) * 60.0).clamp(0.0, 1.0);
        let decay = (-t * 2.0 * decay_scale).exp();
        onset * decay * 0.35 * int
    });
    let whoosh_layer = ((whoosh_src | whoosh_cutoff) >> lowpole()) * whoosh_env;

    // --- Layer 6: Crackle tail (debris/sparks) ---
    let crackle_bp = 5000.0 * pitch;
    let crackle_env = lfo(move |t: f32| -> f32 {
        if t > 1.8 / decay_scale {
            return 0.0;
        }
        let onset = ((t - 0.05) * 20.0).clamp(0.0, 1.0);
        let s1 = (t * 97.3 * std::f32::consts::TAU).sin();
        let s2 = (t * 143.7 * std::f32::consts::TAU).sin();
        let stutter = (s1 * s2).max(0.0);
        let decay = (-t * 2.0 * decay_scale).exp();
        onset * stutter * decay * 0.04 * int
    });
    let crackle_layer = (noise() >> bandpass_hz(crackle_bp, 1.5)) * crackle_env;

    // --- Mix, lowpass, and stereo ---
    let mono_mix = blast_layer + boom_layer + rumble_layer + mid_layer + whoosh_layer
        + crackle_layer;
    // Two-pole lowpass for a steeper roll-off. 20kHz = effectively transparent.
    let graph = (mono_mix >> lowpole_hz(lowpass) >> lowpole_hz(lowpass)) >> split::<U2>();

    if reverb_mix > 0.001 {
        let reverb = reverb2_stereo(0.6, 1.5, 0.5, 1.0, lowpole_hz(2500.0));
        let dry = 1.0 - reverb_mix;
        let wet = reverb_mix;
        let mixed = (graph.clone() * dc((dry, dry))) + (graph >> reverb) * dc((wet, wet));
        Box::new(mixed)
    } else {
        Box::new(graph)
    }
}
