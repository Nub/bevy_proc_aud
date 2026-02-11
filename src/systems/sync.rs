use bevy::prelude::*;

use crate::components::filter::{BandPass, HighPass, LowPass};
use crate::components::synth::{Amplitude, Frequency};
use crate::dsp::graph_builder::SynthParams;
use crate::presets::ear_ringing::{EarRinging, EarRingingParams};
use crate::presets::heartbeat::{Heartbeat, HeartbeatParams};

/// Sync changed synth component values to the audio thread via `ParamHandle` atomics.
pub fn param_sync_system(
    freq_query: Query<(&Frequency, &SynthParams), Changed<Frequency>>,
    amp_query: Query<(&Amplitude, &SynthParams), Changed<Amplitude>>,
    lp_query: Query<(&LowPass, &SynthParams), Changed<LowPass>>,
    hp_query: Query<(&HighPass, &SynthParams), Changed<HighPass>>,
    bp_query: Query<(&BandPass, &SynthParams), Changed<BandPass>>,
) {
    for (freq, params) in &freq_query {
        params.frequency.set(freq.0);
    }
    for (amp, params) in &amp_query {
        params.amplitude.set(amp.0);
    }
    for (lp, params) in &lp_query {
        if let Some(ref cutoff) = params.filter_cutoff {
            cutoff.set(lp.cutoff_hz);
        }
        if let Some(ref res) = params.filter_resonance {
            res.set(lp.resonance);
        }
    }
    for (hp, params) in &hp_query {
        if let Some(ref cutoff) = params.filter_cutoff {
            cutoff.set(hp.cutoff_hz);
        }
    }
    for (bp, params) in &bp_query {
        if let Some(ref cutoff) = params.filter_cutoff {
            cutoff.set(bp.center_hz);
        }
        if let Some(ref bw) = params.filter_resonance {
            bw.set(bp.bandwidth);
        }
    }
}

/// Sync changed `Heartbeat` component values to param handles.
pub fn heartbeat_sync_system(
    query: Query<(&Heartbeat, &HeartbeatParams), Changed<Heartbeat>>,
) {
    for (hb, params) in &query {
        params.rate.set(hb.heart_rate);
        params.intensity.set(hb.intensity);
        params.arrhythmia.set(hb.arrhythmic_strength);
    }
}

/// Sync changed `EarRinging` component values to param handles.
pub fn ear_ringing_sync_system(
    query: Query<(&EarRinging, &EarRingingParams), Changed<EarRinging>>,
) {
    for (er, params) in &query {
        params.intensity.set(er.intensity);
    }
}
