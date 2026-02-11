use bevy::prelude::*;

use crate::dsp::graph_builder::SynthParams;
use crate::dsp::source::ProceduralAudio;
use crate::presets::ear_ringing::EarRingingParams;
use crate::presets::heartbeat::HeartbeatParams;

/// Clean up audio when procedural audio param components are removed.
pub fn audio_cleanup_system(
    mut removed_synth: RemovedComponents<SynthParams>,
    mut removed_heartbeat: RemovedComponents<HeartbeatParams>,
    mut removed_ear_ringing: RemovedComponents<EarRingingParams>,
    mut commands: Commands,
) {
    for entity in removed_synth.read() {
        commands.entity(entity).remove::<AudioPlayer<ProceduralAudio>>();
    }
    for entity in removed_heartbeat.read() {
        commands.entity(entity).remove::<AudioPlayer<ProceduralAudio>>();
    }
    for entity in removed_ear_ringing.read() {
        commands.entity(entity).remove::<AudioPlayer<ProceduralAudio>>();
    }
}
