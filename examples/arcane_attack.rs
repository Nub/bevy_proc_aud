use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_proc_aud::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(BevyProcAudPlugin)
        .insert_resource(Settings {
            intensity: 0.8,
            pitch: 1.0,
            reverb: 0.3,
            lowpass: 20_000.0,
        })
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

#[derive(Resource)]
struct Settings {
    intensity: f32,
    pitch: f32,
    reverb: f32,
    lowpass: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut settings: ResMut<Settings>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Arcane Attack").show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut settings.intensity, 0.0..=1.0).text("Intensity"));
        ui.add(egui::Slider::new(&mut settings.pitch, 0.3..=3.0).logarithmic(true).text("Pitch"));
        ui.add(egui::Slider::new(&mut settings.reverb, 0.0..=1.0).text("Reverb"));
        ui.add(egui::Slider::new(&mut settings.lowpass, 200.0..=20_000.0).logarithmic(true).text("Lowpass"));
        ui.separator();
        if ui.button("Standard").clicked() {
            commands.spawn(ArcaneAttack {
                intensity: settings.intensity,
                pitch_shift: settings.pitch,
                reverb_mix: settings.reverb,
                lowpass: settings.lowpass,
            });
        }
        if ui.button("Deep (0.5x)").clicked() {
            commands.spawn(ArcaneAttack {
                intensity: settings.intensity,
                pitch_shift: 0.5,
                reverb_mix: settings.reverb,
                lowpass: settings.lowpass,
            });
        }
        if ui.button("High (2x)").clicked() {
            commands.spawn(ArcaneAttack {
                intensity: settings.intensity,
                pitch_shift: 2.0,
                reverb_mix: settings.reverb,
                lowpass: settings.lowpass,
            });
        }
        if ui.button("Dry (no reverb)").clicked() {
            commands.spawn(ArcaneAttack {
                intensity: settings.intensity,
                pitch_shift: settings.pitch,
                reverb_mix: 0.0,
                lowpass: settings.lowpass,
            });
        }
    });
    Ok(())
}
