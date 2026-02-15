use bevy::prelude::*;
use fundsp::prelude32::*;

/// One-shot arcane/magic attack sound effect.
///
/// Five layers: shimmering detuned sine cluster, crystalline sparkle,
/// rising frequency sweep, ethereal noise wash, and inharmonic bell-like
/// harmonic cluster. Duration ~0.7s.
///
/// Spawn an entity with this component to trigger the sound.
#[derive(Component, Debug, Clone)]
pub struct ArcaneAttack {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
    /// Pitch multiplier (1.0 = standard, >1 = higher, <1 = deeper).
    pub pitch_shift: f32,
    /// Reverb wet/dry mix (0.0 = dry, 1.0 = fully wet).
    pub reverb_mix: f32,
    /// Low-pass filter cutoff in Hz applied to the whole output (20_000 = effectively off).
    pub lowpass: f32,
}

impl Default for ArcaneAttack {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            pitch_shift: 1.0,
            reverb_mix: 0.3,
            lowpass: 20_000.0,
        }
    }
}

/// Build the arcane attack DSP graph. One-shot, no runtime params.
pub fn build_arcane_attack_graph(aa: &ArcaneAttack) -> Box<dyn AudioUnit> {
    let int = aa.intensity;
    let pitch = aa.pitch_shift;
    let reverb_mix = aa.reverb_mix;
    let lowpass = aa.lowpass;

    // --- Layer 1: Shimmer Core ---
    // 6 detuned sines in two clusters around 880Hz and 1320Hz with +/-5 cent detune.
    let base_a = 880.0 * pitch;
    let base_b = 1320.0 * pitch;
    // 5 cents ≈ multiply by 2^(5/1200) ≈ 1.002893
    let detune_up = 1.002893_f32;
    let detune_dn = 1.0 / detune_up;
    let shimmer_env = lfo(move |t: f32| -> f32 {
        if t > 0.55 {
            return 0.0;
        }
        let attack = (t * 40.0).min(1.0);
        let decay = (-t * 5.5).exp();
        attack * decay * 0.15 * int
    });
    let shimmer_layer = (sine_hz(base_a)
        + sine_hz(base_a * detune_up)
        + sine_hz(base_a * detune_dn)
        + sine_hz(base_b)
        + sine_hz(base_b * detune_up)
        + sine_hz(base_b * detune_dn))
        * dc(1.0 / 6.0)
        * shimmer_env;

    // --- Layer 2: Crystalline Sparkle ---
    // Bandpassed noise with granular stuttering envelope.
    let sparkle_center = 6000.0 * pitch;
    let sparkle_env = lfo(move |t: f32| -> f32 {
        if t > 0.6 {
            return 0.0;
        }
        let onset = (t * 80.0).min(1.0);
        let decay = (-t * 4.5).exp();
        // Granular stuttering via multiplied sines
        let s1 = (t * 73.0 * std::f32::consts::TAU).sin();
        let s2 = (t * 113.0 * std::f32::consts::TAU).sin();
        let stutter = (s1 * s2).max(0.0);
        onset * decay * stutter * 0.25 * int
    });
    let sparkle_layer = (noise() >> bandpass_hz(sparkle_center, 2.0)) * sparkle_env;

    // --- Layer 3: Rising Sweep ---
    // LFO-driven pitch sweep with FM modulation, 300-1800Hz.
    let sweep_lo = 300.0 * pitch;
    let sweep_hi = 1800.0 * pitch;
    let sweep_freq = lfo(move |t: f32| -> f32 {
        if t > 0.45 {
            return 0.0;
        }
        // Rising sweep: low to high over 0.45 seconds
        let ratio = t / 0.45;
        sweep_lo + (sweep_hi - sweep_lo) * ratio
    });
    let sweep_env = lfo(move |t: f32| -> f32 {
        if t > 0.45 {
            return 0.0;
        }
        let attack = (t * 30.0).min(1.0);
        let decay = (-(t - 0.35).max(0.0) * 20.0).exp() * attack;
        decay * 0.12 * int
    });
    // FM: modulate the sweep with a small sine vibrato
    let fm_mod = sine_hz(7.0 * pitch) * dc(30.0 * pitch);
    let sweep_layer = ((sweep_freq + fm_mod) >> sine()) * sweep_env;

    // --- Layer 4: Ethereal Wash ---
    // Noise through opening/closing lowpole, 200-1200Hz.
    let wash_lo = 200.0 * pitch;
    let wash_hi = 1200.0 * pitch;
    let wash_cutoff = lfo(move |t: f32| -> f32 {
        if t > 0.6 {
            return wash_lo;
        }
        // Open up then close: bell curve
        let x = t / 0.6; // 0..1
        let curve = (-(x - 0.4).powi(2) * 12.0).exp();
        wash_lo + (wash_hi - wash_lo) * curve
    });
    let wash_env = lfo(move |t: f32| -> f32 {
        if t > 0.6 {
            return 0.0;
        }
        let attack = (t * 20.0).min(1.0);
        let decay = (-t * 3.5).exp();
        attack * decay * 0.30 * int
    });
    let wash_layer = ((noise() | wash_cutoff) >> lowpole()) * wash_env;

    // --- Layer 5: Harmonic Cluster ---
    // Inharmonic sine partials (bell-like), 1320-3200Hz range.
    let h1 = 1320.0 * pitch;
    let h2 = 1720.0 * pitch;
    let h3 = 2150.0 * pitch;
    let h4 = 2680.0 * pitch;
    let h5 = 3200.0 * pitch;
    let cluster_env = lfo(move |t: f32| -> f32 {
        if t > 0.4 {
            return 0.0;
        }
        let attack = (t * 120.0).min(1.0);
        let decay = (-t * 8.0).exp();
        attack * decay * 0.08 * int
    });
    let cluster_layer = (sine_hz(h1)
        + sine_hz(h2) * dc(0.8)
        + sine_hz(h3) * dc(0.6)
        + sine_hz(h4) * dc(0.4)
        + sine_hz(h5) * dc(0.2))
        * dc(1.0 / 3.0)
        * cluster_env;

    // --- Mix, lowpass, and stereo ---
    let mono_mix =
        shimmer_layer + sparkle_layer + sweep_layer + wash_layer + cluster_layer;
    // Two-pole lowpass for a steeper roll-off. 20kHz = effectively transparent.
    let graph = (mono_mix >> lowpole_hz(lowpass) >> lowpole_hz(lowpass)) >> split::<U2>();

    if reverb_mix > 0.001 {
        let reverb = reverb2_stereo(0.5, 1.0, 0.7, 1.0, lowpole_hz(3500.0));
        let dry = 1.0 - reverb_mix;
        let wet = reverb_mix;
        let mixed = (graph.clone() * dc((dry, dry))) + (graph >> reverb) * dc((wet, wet));
        Box::new(mixed)
    } else {
        Box::new(graph)
    }
}
