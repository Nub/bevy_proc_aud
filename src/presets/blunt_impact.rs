use bevy::prelude::*;
use fundsp::prelude32::*;

/// One-shot blunt impact sound effect — mace, hammer, or club striking a body.
/// Three layers: impact crack, body thud, metallic clang.
///
/// Spawn an entity with this component to trigger the sound.
/// The sound plays for ~0.3s then goes silent.
#[derive(Component, Debug, Clone)]
pub struct BluntImpact {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
    /// Pitch multiplier (1.0 = normal, <1 = lower, >1 = higher). Use for variance.
    pub pitch_shift: f32,
    /// Reverb wet/dry mix (0.0 = dry, 1.0 = fully wet). Adds cave-like ambience.
    pub reverb_mix: f32,
}

impl Default for BluntImpact {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            pitch_shift: 1.0,
            reverb_mix: 0.0,
        }
    }
}

/// Build the blunt impact DSP graph. One-shot, no runtime params.
pub fn build_blunt_impact_graph(bi: &BluntImpact) -> Box<dyn AudioUnit> {
    let intensity = bi.intensity;
    let pitch = bi.pitch_shift;
    let reverb_mix = bi.reverb_mix;

    let mut net = Net::new(0, 2);

    // --- Layer 1: Impact crack (punchy broadband noise burst) ---
    let crack_cutoff = 5000.0 * pitch;
    let crack_src_id = net.push(Box::new(noise() >> lowpole_hz(crack_cutoff)));

    let crack_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.1 {
            return 0.0;
        }
        let attack = (t * 500.0).min(1.0);
        let decay = (-t * 35.0).exp();
        attack * decay * 0.5 * intensity
    })));

    let crack_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(crack_src_id, 0, crack_id, 0);
    net.connect(crack_env_id, 0, crack_id, 1);

    // --- Layer 2: Body thud (low-frequency weight) ---
    let thud_lo = 45.0 * pitch;
    let thud_hi = 90.0 * pitch;
    let thud_src_id = net.push(Box::new(
        dc(thud_lo) >> sine() + dc(thud_hi) >> sine(),
    ));

    let thud_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.15 {
            return 0.0;
        }
        let attack = (t * 200.0).min(1.0);
        let decay = (-t * 20.0).exp();
        attack * decay * 0.35 * intensity
    })));

    let thud_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(thud_src_id, 0, thud_id, 0);
    net.connect(thud_env_id, 0, thud_id, 1);

    // --- Layer 3: Weapon clang (inharmonic sine cluster) ---
    let c1 = 780.0 * pitch;
    let c2 = 1850.0 * pitch;
    let c3 = 3100.0 * pitch;
    let c4 = 4700.0 * pitch;
    let clang_src_id = net.push(Box::new(
        dc(c1) >> sine()
            + dc(c2) >> sine()
            + dc(c3) >> sine()
            + dc(c4) >> sine(),
    ));

    let clang_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.2 {
            return 0.0;
        }
        let attack = (t * 500.0).min(1.0);
        let decay = (-t * 18.0).exp();
        attack * decay * 0.08 * intensity
    })));

    let clang_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(clang_src_id, 0, clang_id, 0);
    net.connect(clang_env_id, 0, clang_id, 1);

    // --- Mix ---
    let mix_id = net.push(Box::new(map(|f: &Frame<f32, U3>| -> f32 {
        f[0] + f[1] + f[2]
    })));
    net.connect(crack_id, 0, mix_id, 0);
    net.connect(thud_id, 0, mix_id, 1);
    net.connect(clang_id, 0, mix_id, 2);

    let split_id = net.push(Box::new(split::<U2>()));
    net.connect(mix_id, 0, split_id, 0);
    net.connect_output(split_id, 0, 0);
    net.connect_output(split_id, 1, 1);

    if reverb_mix > 0.001 {
        let reverb = reverb2_stereo(0.4, 0.8, 0.5, 1.0, lowpole_hz(4000.0));
        let dry = 1.0 - reverb_mix;
        let wet = reverb_mix;
        // dry/wet crossfade: stack dry + reverbed, mix per channel
        let mixed = (net.clone() * dc((dry, dry))) + (net >> reverb) * dc((wet, wet));
        Box::new(mixed)
    } else {
        Box::new(net)
    }
}
