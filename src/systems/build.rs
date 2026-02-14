use bevy::prelude::*;

use crate::components::effect::{Delay, Distortion, Reverb};
use crate::components::filter::{BandPass, HighPass, LowPass};
use crate::components::synth::{Amplitude, Frequency, OscillatorType, Synth};
use crate::dsp::graph_builder::build_synth_graph;
use crate::dsp::source::ProceduralAudio;
use crate::presets::blunt_impact::{build_blunt_impact_graph, BluntImpact};
use crate::presets::ear_ringing::{build_ear_ringing_graph, EarRinging};
use crate::presets::heartbeat::{build_heartbeat_graph, Heartbeat};
use crate::presets::sword_slash::{build_sword_slash_graph, SwordSlash};

const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

/// Build DSP graphs for newly-added `Synth` entities.
pub fn graph_build_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            Option<&OscillatorType>,
            Option<&Frequency>,
            Option<&Amplitude>,
            Option<&LowPass>,
            Option<&HighPass>,
            Option<&BandPass>,
            Option<&Reverb>,
            Option<&Delay>,
            Option<&Distortion>,
        ),
        Added<Synth>,
    >,
    mut assets: ResMut<Assets<ProceduralAudio>>,
) {
    for (entity, osc, freq, amp, lp, hp, bp, reverb, delay, dist) in &query {
        let osc_type = osc.copied().unwrap_or_default();
        let frequency = freq.copied().unwrap_or_default();
        let amplitude = amp.copied().unwrap_or_default();

        let (graph, params) = build_synth_graph(
            &osc_type, &frequency, &amplitude, lp, hp, bp, reverb, delay, dist,
        );

        let audio = ProceduralAudio::new(graph, SAMPLE_RATE, CHANNELS);
        let handle = assets.add(audio);

        commands.entity(entity).insert((
            AudioPlayer::<ProceduralAudio>(handle),
            params,
        ));
    }
}

/// Build DSP graph for newly-added `Heartbeat` entities.
pub fn heartbeat_build_system(
    mut commands: Commands,
    query: Query<(Entity, &Heartbeat), Added<Heartbeat>>,
    mut assets: ResMut<Assets<ProceduralAudio>>,
) {
    for (entity, hb) in &query {
        let (graph, params) = build_heartbeat_graph(hb);
        let audio = ProceduralAudio::new(graph, SAMPLE_RATE, CHANNELS);
        let handle = assets.add(audio);

        commands.entity(entity).insert((
            AudioPlayer::<ProceduralAudio>(handle),
            params,
        ));
    }
}

/// Build DSP graph for newly-added `SwordSlash` entities.
pub fn sword_slash_build_system(
    mut commands: Commands,
    query: Query<(Entity, &SwordSlash), Added<SwordSlash>>,
    mut assets: ResMut<Assets<ProceduralAudio>>,
) {
    for (entity, ss) in &query {
        let graph = build_sword_slash_graph(ss);
        let audio = ProceduralAudio::new(graph, SAMPLE_RATE, CHANNELS);
        let handle = assets.add(audio);

        commands
            .entity(entity)
            .insert(AudioPlayer::<ProceduralAudio>(handle));
    }
}

/// Build DSP graph for newly-added `BluntImpact` entities.
pub fn blunt_impact_build_system(
    mut commands: Commands,
    query: Query<(Entity, &BluntImpact), Added<BluntImpact>>,
    mut assets: ResMut<Assets<ProceduralAudio>>,
) {
    for (entity, bi) in &query {
        let graph = build_blunt_impact_graph(bi);
        let audio = ProceduralAudio::new(graph, SAMPLE_RATE, CHANNELS);
        let handle = assets.add(audio);

        commands
            .entity(entity)
            .insert(AudioPlayer::<ProceduralAudio>(handle));
    }
}

/// Build DSP graph for newly-added `EarRinging` entities.
pub fn ear_ringing_build_system(
    mut commands: Commands,
    query: Query<(Entity, &EarRinging), Added<EarRinging>>,
    mut assets: ResMut<Assets<ProceduralAudio>>,
) {
    for (entity, er) in &query {
        let (graph, params) = build_ear_ringing_graph(er);
        let audio = ProceduralAudio::new(graph, SAMPLE_RATE, CHANNELS);
        let handle = assets.add(audio);

        commands.entity(entity).insert((
            AudioPlayer::<ProceduralAudio>(handle),
            params,
        ));
    }
}
