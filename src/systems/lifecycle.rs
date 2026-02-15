use bevy::prelude::*;

use crate::components::lifetime::OneShotLifetime;
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

/// Despawn one-shot audio entities after their sound has finished.
pub fn oneshot_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut OneShotLifetime)>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.elapsed += dt;
        if lifetime.elapsed >= lifetime.duration {
            commands.entity(entity).despawn();
        }
    }
}
