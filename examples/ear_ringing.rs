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
    commands.spawn(EarRinging { intensity: 0.3 });
}

fn ui_system(mut contexts: EguiContexts, mut query: Query<&mut EarRinging>) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Ear Ringing Controls").show(ctx, |ui| {
        for mut er in &mut query {
            ui.add(egui::Slider::new(&mut er.intensity, 0.0..=1.0).text("Intensity"));
        }
    });
    Ok(())
}
