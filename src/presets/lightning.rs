use bevy::prelude::*;
use fundsp::prelude32::*;

/// One-shot electrical zap — sustained buzzy arc discharge.
///
/// Three layers: buzzy sawtooth-like FM tone with downward pitch sweep for
/// the core "zzzzap", amplitude-modulated noise for crackle/sizzle, and a
/// bright transient at the start. Duration ~0.4s.
///
/// Spawn an entity with this component to trigger the sound.
#[derive(Component, Debug, Clone)]
pub struct LightningZap {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
    /// Pitch multiplier (1.0 = normal, <1 = lower, >1 = higher).
    pub pitch_shift: f32,
    /// Reverb wet/dry mix (0.0 = dry, 1.0 = fully wet).
    pub reverb_mix: f32,
}

impl Default for LightningZap {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            pitch_shift: 1.0,
            reverb_mix: 0.0,
        }
    }
}

/// Build the lightning zap DSP graph. One-shot, no runtime params.
pub fn build_lightning_zap_graph(zap: &LightningZap) -> Box<dyn AudioUnit> {
    let int = zap.intensity;
    let pitch = zap.pitch_shift;
    let reverb_mix = zap.reverb_mix;

    // Reference analysis: spectral centroid ~5400Hz, 95%+ energy above 2kHz,
    // erratic stuttering envelope, ~500ms duration, peak RMS ~0.3.

    // --- Layer 1: Core zap (bandpassed noise at ~5kHz) ---
    // Wide bandpass noise centered around 5kHz — this is the main sizzle.
    // Chaotically stuttering envelope from overlapping inharmonic sine gates.
    let bp1 = 5000.0 * pitch;
    let zap_env = lfo(move |t: f32| -> f32 {
        if t > 0.55 {
            return 0.0;
        }
        // Chaotic stutter: product of sines at inharmonic rates
        // creates pseudo-random gating (arc making/breaking contact)
        let s1 = (t * 127.3 * std::f32::consts::TAU).sin();
        let s2 = (t * 89.7 * std::f32::consts::TAU).sin();
        let s3 = (t * 211.1 * std::f32::consts::TAU).sin();
        let stutter = (s1 * s2 * s3).max(0.0); // half-wave rectified product
        let overall = (-t * 3.5).exp();
        stutter * overall * 0.65 * int
    });
    let zap_layer = (noise() >> bandpass_hz(bp1, 1.5)) * zap_env;

    // --- Layer 2: High sizzle (noise above 5kHz) ---
    // Additional high-frequency content for brightness and air.
    let bp2 = 7000.0 * pitch;
    let sizzle_env = lfo(move |t: f32| -> f32 {
        if t > 0.5 {
            return 0.0;
        }
        // Different stutter pattern (different frequencies)
        let s1 = (t * 173.9 * std::f32::consts::TAU).sin();
        let s2 = (t * 67.3 * std::f32::consts::TAU).sin();
        let stutter = (s1 * s2).max(0.0);
        let overall = (-t * 4.0).exp();
        stutter * overall * 0.4 * int
    });
    let sizzle_layer = (noise() >> bandpass_hz(bp2, 1.0)) * sizzle_env;

    // --- Layer 3: Mid crackle (~3-4kHz) ---
    // Fills out the spectrum in the 2-5kHz range.
    let bp3 = 3500.0 * pitch;
    let mid_env = lfo(move |t: f32| -> f32 {
        if t > 0.5 {
            return 0.0;
        }
        let s1 = (t * 151.7 * std::f32::consts::TAU).sin();
        let s2 = (t * 103.3 * std::f32::consts::TAU).sin();
        let s3 = (t * 197.9 * std::f32::consts::TAU).sin();
        let stutter = (s1 * s2 * s3).max(0.0);
        let overall = (-t * 3.0).exp();
        stutter * overall * 0.35 * int
    });
    let mid_layer = (noise() >> bandpass_hz(bp3, 1.5)) * mid_env;

    // --- Mix and stereo ---
    let graph = (zap_layer + sizzle_layer + mid_layer) >> split::<U2>();

    if reverb_mix > 0.001 {
        let reverb = reverb2_stereo(0.2, 0.4, 0.3, 1.0, lowpole_hz(8000.0));
        let dry = 1.0 - reverb_mix;
        let wet = reverb_mix;
        let mixed = (graph.clone() * dc((dry, dry))) + (graph >> reverb) * dc((wet, wet));
        Box::new(mixed)
    } else {
        Box::new(graph)
    }
}

/// One-shot lightning strike — massive thunder boom with electrical crack.
///
/// Four layers: bright initial crack, huge low-frequency boom, mid body,
/// and electrical crackle. The boom dominates. Duration ~2.5s.
///
/// Spawn an entity with this component to trigger the sound.
#[derive(Component, Debug, Clone)]
pub struct LightningStrike {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
    /// Pitch multiplier (1.0 = normal, <1 = lower, >1 = higher).
    pub pitch_shift: f32,
    /// Reverb wet/dry mix (0.0 = dry, 1.0 = fully wet). Adds distant-storm ambience.
    pub reverb_mix: f32,
}

impl Default for LightningStrike {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            pitch_shift: 1.0,
            reverb_mix: 0.15,
        }
    }
}

/// Build the lightning strike DSP graph. One-shot, no runtime params.
pub fn build_lightning_strike_graph(ls: &LightningStrike) -> Box<dyn AudioUnit> {
    let int = ls.intensity;
    let pitch = ls.pitch_shift;
    let reverb_mix = ls.reverb_mix;

    // --- Layer 1: Initial crack (bright broadband transient) ---
    // Full-spectrum noise burst — the sharp CRACK at the instant of the strike.
    let crack_env = lfo(move |t: f32| -> f32 {
        if t > 0.15 {
            return 0.0;
        }
        let attack = (t * 5000.0).min(1.0);
        let decay = (-t * 20.0).exp();
        attack * decay * 0.5 * int
    });
    let crack_layer = noise() * crack_env;

    // --- Layer 2: Low boom (dominant thunder body) ---
    // Heavy low-passed noise — this is the chest-thumping BOOM.
    // Two cascaded lowpole filters for steep rolloff. High amplitude.
    let boom_cutoff = 80.0 * pitch;
    let boom_env = lfo(move |t: f32| -> f32 {
        if t > 2.5 {
            return 0.0;
        }
        // Fast attack, long sustain/decay for rolling thunder
        let attack = (t * 100.0).min(1.0);
        let decay = (-t * 1.2).exp();
        attack * decay * 0.7 * int
    });
    let boom_layer =
        (noise() >> lowpole_hz(boom_cutoff) >> lowpole_hz(boom_cutoff)) * boom_env;

    // --- Layer 3: Mid body (fills out the thunder) ---
    // Mid-frequency noise gives body between crack and boom.
    let mid_cutoff = 400.0 * pitch;
    let mid_env = lfo(move |t: f32| -> f32 {
        if t > 1.5 {
            return 0.0;
        }
        let attack = (t * 200.0).min(1.0);
        let decay = (-t * 2.0).exp();
        attack * decay * 0.3 * int
    });
    let mid_layer = (noise() >> lowpole_hz(mid_cutoff)) * mid_env;

    // --- Layer 4: Electrical crackle (FM chaos, secondary) ---
    // Adds the electrical sizzle on top of the boom.
    let c1_carrier = 1800.0 * pitch;
    let c1_mod = 1270.0 * pitch;
    let fm1 = (dc(c1_carrier)
        + sine_hz(c1_mod)
            * lfo(move |t: f32| -> f32 { 30.0 * c1_mod * (-t * 6.0).exp() }))
        >> sine();

    let crackle_env = lfo(move |t: f32| -> f32 {
        if t > 0.8 {
            return 0.0;
        }
        let attack = (t * 1000.0).min(1.0);
        let decay = (-t * 4.0).exp();
        attack * decay * 0.06 * int
    });
    let crackle_layer = fm1 * crackle_env;

    // --- Mix and stereo ---
    let graph = (crack_layer + boom_layer + mid_layer + crackle_layer) >> split::<U2>();

    if reverb_mix > 0.001 {
        let reverb = reverb2_stereo(0.6, 1.5, 0.5, 1.0, lowpole_hz(2000.0));
        let dry = 1.0 - reverb_mix;
        let wet = reverb_mix;
        let mixed = (graph.clone() * dc((dry, dry))) + (graph >> reverb) * dc((wet, wet));
        Box::new(mixed)
    } else {
        Box::new(graph)
    }
}
