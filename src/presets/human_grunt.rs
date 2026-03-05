use bevy::prelude::*;
use fundsp::prelude32::*;

/// One-shot human grunt sound effect — voiced male pain/effort vocalization.
/// Sawtooth excitation with glottal tilt, filtered through wide formant
/// resonances with pitch drop, producing a deep throaty "UHHhh" grunt.
///
/// Spawn an entity with this component to trigger the sound.
#[derive(Component, Debug, Clone)]
pub struct HumanGrunt {
    /// Overall intensity (0.0–1.0).
    pub intensity: f32,
    /// Pitch multiplier (1.0 = normal, <1 = deeper grunt, >1 = higher). Use for variance.
    pub pitch_shift: f32,
    /// Reverb wet/dry mix (0.0 = dry, 1.0 = fully wet). Adds cave-like ambience.
    pub reverb_mix: f32,
}

impl Default for HumanGrunt {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            pitch_shift: 1.0,
            reverb_mix: 0.0,
        }
    }
}

/// Build the human grunt DSP graph. One-shot, no runtime params.
pub fn build_human_grunt_graph(grunt: &HumanGrunt) -> Box<dyn AudioUnit> {
    let intensity = grunt.intensity;
    let pitch = grunt.pitch_shift;
    let reverb_mix = grunt.reverb_mix;

    let mut net = Net::new(0, 2);

    // ── Voiced excitation with pitch drop ──
    // Male fundamental (~95 Hz), drops ~25% over the grunt.
    let f0 = 95.0 * pitch;

    let pitch_lfo_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        f0 * (1.0 - t * 0.6).max(0.75)
    })));
    let saw_id = net.push(Box::new(saw()));
    net.connect(pitch_lfo_id, 0, saw_id, 0);

    // Detuned second saw for vocal roughness
    let pitch_lfo2_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        f0 * 1.012 * (1.0 - t * 0.6).max(0.75)
    })));
    let saw2_id = net.push(Box::new(saw()));
    net.connect(pitch_lfo2_id, 0, saw2_id, 0);

    let saw_mix_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * 0.6 + f[1] * 0.4
    })));
    net.connect(saw_id, 0, saw_mix_id, 0);
    net.connect(saw2_id, 0, saw_mix_id, 1);

    // ── Glottal spectral tilt ──
    // Heavy lowpass to kill the buzziness — real glottal output is dark.
    let glottal_lp_id = net.push(Box::new(lowpole_hz(1400.0 * pitch)));
    net.connect(saw_mix_id, 0, glottal_lp_id, 0);

    // ── Formant filters — wide Q for natural sound ──
    let fan_id = net.push(Box::new(split::<U3>()));
    net.connect(glottal_lp_id, 0, fan_id, 0);

    // F1 (~400 Hz) — deep "UH" vowel body. Wide Q avoids ringing artifacts.
    let f1_id = net.push(Box::new(bandpass_hz(400.0 * pitch, 4.0)));
    net.connect(fan_id, 0, f1_id, 0);

    // F2 (~800 Hz) — secondary resonance
    let f2_id = net.push(Box::new(bandpass_hz(800.0 * pitch, 4.0)));
    net.connect(fan_id, 1, f2_id, 0);

    // F3 (~2000 Hz) — subtle presence
    let f3_id = net.push(Box::new(bandpass_hz(2000.0 * pitch, 3.0)));
    net.connect(fan_id, 2, f3_id, 0);

    // Mix formants — F1 dominates heavily for deep grunt character.
    let formant_mix_id = net.push(Box::new(map(|f: &Frame<f32, U3>| -> f32 {
        f[0] * 0.6 + f[1] * 0.3 + f[2] * 0.1
    })));
    net.connect(f1_id, 0, formant_mix_id, 0);
    net.connect(f2_id, 0, formant_mix_id, 1);
    net.connect(f3_id, 0, formant_mix_id, 2);

    // Voiced envelope: fast attack, brief peak, gradual trail-off
    let voice_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.4 {
            return 0.0;
        }
        let attack = (t * 200.0).min(1.0);
        let decay = if t < 0.08 {
            1.0
        } else {
            (-(t - 0.08) * 7.0).exp()
        };
        attack * decay * 1.8 * intensity
    })));

    let voice_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(formant_mix_id, 0, voice_id, 0);
    net.connect(voice_env_id, 0, voice_id, 1);

    // ── Breathiness ──
    // Broad noise in the low-mid range — sounds like strained air.
    let breath_src_id = net.push(Box::new(
        noise() >> lowpole_hz(1200.0 * pitch) >> highpole_hz(200.0 * pitch),
    ));

    let breath_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.25 {
            return 0.0;
        }
        let attack = (t * 200.0).min(1.0);
        let decay = (-t * 10.0).exp();
        attack * decay * 0.15 * intensity
    })));

    let breath_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(breath_src_id, 0, breath_id, 0);
    net.connect(breath_env_id, 0, breath_id, 1);

    // ── Sub chest thump ──
    let sub_freq = 65.0 * pitch;
    let sub_src_id = net.push(Box::new(dc(sub_freq) >> sine()));

    let sub_env_id = net.push(Box::new(lfo(move |t: f32| -> f32 {
        if t > 0.12 {
            return 0.0;
        }
        let attack = (t * 300.0).min(1.0);
        let decay = (-t * 25.0).exp();
        attack * decay * 0.2 * intensity
    })));

    let sub_id = net.push(Box::new(map(|f: &Frame<f32, U2>| -> f32 {
        f[0] * f[1]
    })));
    net.connect(sub_src_id, 0, sub_id, 0);
    net.connect(sub_env_id, 0, sub_id, 1);

    // ── Final mix ──
    let mix_id = net.push(Box::new(map(|f: &Frame<f32, U3>| -> f32 {
        f[0] + f[1] + f[2]
    })));
    net.connect(voice_id, 0, mix_id, 0);
    net.connect(breath_id, 0, mix_id, 1);
    net.connect(sub_id, 0, mix_id, 2);

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
