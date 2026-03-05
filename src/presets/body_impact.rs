use bevy::prelude::*;
use fundsp::prelude32::*;

/// One-shot body impact sound effect — fleshy thud when a body is struck.
/// Three layers: broadband crack, sub thump, filtered burst.
///
/// Spawn an entity with this component to trigger the sound.
/// The sound plays for ~0.3s then goes silent.
#[derive(Component, Debug, Clone)]
pub struct BodyImpact {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
    /// Pitch multiplier (1.0 = normal, <1 = lower, >1 = higher). Use for variance.
    pub pitch_shift: f32,
    /// Reverb wet/dry mix (0.0 = dry, 1.0 = fully wet). Adds cave-like ambience.
    pub reverb_mix: f32,
}

impl Default for BodyImpact {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            pitch_shift: 1.0,
            reverb_mix: 0.0,
        }
    }
}

/// Build the body impact DSP graph. One-shot, no runtime params.
pub fn build_body_impact_graph(bi: &BodyImpact) -> Box<dyn AudioUnit> {
    let intensity = bi.intensity;
    let pitch = bi.pitch_shift;
    let reverb_mix = bi.reverb_mix;

    let mut net = Net::new(0, 2);

    // --- Layer 1: Broadband crack (bandpassed noise) ---
    let f1_center = 300.0 * pitch;
    let f1_src_id = net.push(Box::new(
        noise() >> bandpass_hz(f1_center, 5.0),
    ));

    let f2_center = 800.0 * pitch;
    let f2_src_id = net.push(Box::new(
        noise() >> bandpass_hz(f2_center, 5.0),
    ));

    let formant_mix_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * 0.6 + f[1] * 0.4
    })));
    net.connect(f1_src_id, 0, formant_mix_id, 0);
    net.connect(f2_src_id, 0, formant_mix_id, 1);

    let formant_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.25 {
            return 0.0;
        }
        let attack = (t * 300.0).min(1.0);
        let decay = (-t * 15.0).exp();
        attack * decay * 0.6 * intensity
    })));

    let formant_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(formant_mix_id, 0, formant_id, 0);
    net.connect(formant_env_id, 0, formant_id, 1);

    // --- Layer 2: Sub thump ---
    let sub_freq = 90.0 * pitch;
    let sub_src_id = net.push(Box::new(dc(sub_freq) >> sine()));

    let sub_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.12 {
            return 0.0;
        }
        let attack = (t * 400.0).min(1.0);
        let decay = (-t * 30.0).exp();
        attack * decay * 0.3 * intensity
    })));

    let sub_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(sub_src_id, 0, sub_id, 0);
    net.connect(sub_env_id, 0, sub_id, 1);

    // --- Layer 3: High-frequency burst ---
    let breath_cutoff = 3000.0 * pitch;
    let breath_src_id = net.push(Box::new(
        noise() >> highpole_hz(breath_cutoff) >> lowpole_hz(breath_cutoff * 1.5),
    ));

    let breath_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.08 {
            return 0.0;
        }
        let attack = (t * 600.0).min(1.0);
        let decay = (-t * 50.0).exp();
        attack * decay * 0.15 * intensity
    })));

    let breath_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(breath_src_id, 0, breath_id, 0);
    net.connect(breath_env_id, 0, breath_id, 1);

    // --- Mix ---
    let mix_id = net.push(Box::new(map(|f: &Frame<f32, U3>| -> f32 {
        f[0] + f[1] + f[2]
    })));
    net.connect(formant_id, 0, mix_id, 0);
    net.connect(sub_id, 0, mix_id, 1);
    net.connect(breath_id, 0, mix_id, 2);

    let split_id = net.push(Box::new(split::<U2>()));
    net.connect(mix_id, 0, split_id, 0);
    net.connect_output(split_id, 0, 0);
    net.connect_output(split_id, 1, 1);

    if reverb_mix > 0.001 {
        let reverb = reverb2_stereo(0.4, 0.8, 0.5, 1.0, lowpole_hz(4000.0));
        let dry = 1.0 - reverb_mix;
        let wet = reverb_mix;
        let mixed = (net.clone() * dc((dry, dry))) + (net >> reverb) * dc((wet, wet));
        Box::new(mixed)
    } else {
        Box::new(net)
    }
}
