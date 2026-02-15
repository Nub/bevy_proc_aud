use bevy::audio::AddAudioSource;
use bevy::prelude::*;

use crate::dsp::source::ProceduralAudio;
use crate::systems::build::{
    blunt_impact_build_system, ear_ringing_build_system, graph_build_system,
    heartbeat_build_system, lightning_strike_build_system, lightning_zap_build_system,
    sword_slash_build_system,
};
use crate::systems::lifecycle::{audio_cleanup_system, oneshot_lifetime_system};
use crate::systems::sync::{ear_ringing_sync_system, heartbeat_sync_system, param_sync_system};

/// Main plugin for bevy_proc_aud.
///
/// Registers the `ProceduralAudio` asset type and all build/sync/lifecycle systems.
pub struct BevyProcAudPlugin;

impl Plugin for BevyProcAudPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<ProceduralAudio>()
            .add_systems(
                Update,
                (
                    // Build systems (react to Added<T>).
                    graph_build_system,
                    heartbeat_build_system,
                    ear_ringing_build_system,
                    sword_slash_build_system,
                    blunt_impact_build_system,
                    lightning_zap_build_system,
                    lightning_strike_build_system,
                    // Sync systems (react to Changed<T>).
                    param_sync_system,
                    heartbeat_sync_system,
                    ear_ringing_sync_system,
                    // Lifecycle.
                    audio_cleanup_system,
                    oneshot_lifetime_system,
                ),
            );
    }
}
