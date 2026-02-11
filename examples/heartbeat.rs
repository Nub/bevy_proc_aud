use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_proc_aud::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(BevyProcAudPlugin)
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Heartbeat {
        heart_rate: 72.0,
        arrhythmic_strength: 0.0,
        intensity: 0.5,
    });
}

fn ui_system(mut contexts: EguiContexts, mut query: Query<&mut Heartbeat>) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Heartbeat Controls").show(ctx, |ui| {
        for mut hb in &mut query {
            ui.add(egui::Slider::new(&mut hb.heart_rate, 30.0..=220.0).text("Heart Rate (BPM)"));
            ui.add(
                egui::Slider::new(&mut hb.arrhythmic_strength, 0.0..=1.0)
                    .text("Arrhythmia"),
            );
            ui.add(egui::Slider::new(&mut hb.intensity, 0.0..=1.0).text("Intensity"));
        }
    });
    Ok(())
}
