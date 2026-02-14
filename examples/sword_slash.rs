use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_proc_aud::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(BevyProcAudPlugin)
        .insert_resource(Settings { intensity: 0.8 })
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

#[derive(Resource)]
struct Settings {
    intensity: f32,
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
    egui::Window::new("Weapon Sounds").show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut settings.intensity, 0.0..=1.0).text("Intensity"));
        ui.separator();
        if ui.button("Sword Slash").clicked() {
            commands.spawn(SwordSlash {
                intensity: settings.intensity,
            });
        }
        if ui.button("Blunt Impact").clicked() {
            commands.spawn(BluntImpact {
                intensity: settings.intensity,
            });
        }
    });
    Ok(())
}
