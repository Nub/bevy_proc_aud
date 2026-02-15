use bevy::prelude::*;

/// Marks a one-shot audio entity for automatic despawn after a fixed duration.
///
/// Inserted by build systems for one-shot presets (SwordSlash, BluntImpact,
/// LightningZap, LightningStrike). The lifecycle system ticks the elapsed
/// time and despawns the entity once it exceeds `duration`.
#[derive(Component)]
pub struct OneShotLifetime {
    pub duration: f32,
    pub elapsed: f32,
}

impl OneShotLifetime {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
        }
    }
}
