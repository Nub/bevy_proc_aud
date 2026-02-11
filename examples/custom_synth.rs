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
    commands.spawn((
        Synth,
        OscillatorType::Saw,
        Frequency(440.0),
        Amplitude(0.3),
        LowPass {
            cutoff_hz: 2000.0,
            resonance: 2.0,
        },
        Reverb {
            room_size: 0.5,
            decay_time: 1.5,
            damping: 0.3,
            mix: 0.3,
        },
    ));
}

fn ui_system(
    mut contexts: EguiContexts,
    mut query: Query<(&mut Frequency, &mut Amplitude, &mut LowPass, &mut Reverb)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Synth Controls").show(ctx, |ui| {
        for (mut freq, mut amp, mut lp, mut rev) in &mut query {
            ui.heading("Oscillator");
            ui.add(
                egui::Slider::new(&mut freq.0, 20.0..=8000.0)
                    .logarithmic(true)
                    .text("Frequency (Hz)"),
            );
            ui.add(egui::Slider::new(&mut amp.0, 0.0..=1.0).text("Amplitude"));

            ui.separator();
            ui.heading("Filter");
            ui.add(
                egui::Slider::new(&mut lp.cutoff_hz, 20.0..=20000.0)
                    .logarithmic(true)
                    .text("Cutoff (Hz)"),
            );
            ui.add(egui::Slider::new(&mut lp.resonance, 0.1..=10.0).text("Resonance"));

            ui.separator();
            ui.heading("Reverb");
            ui.add(egui::Slider::new(&mut rev.room_size, 0.0..=1.0).text("Room Size"));
            ui.add(egui::Slider::new(&mut rev.decay_time, 0.1..=10.0).text("Decay Time"));
            ui.add(egui::Slider::new(&mut rev.damping, 0.0..=1.0).text("Damping"));
            ui.add(egui::Slider::new(&mut rev.mix, 0.0..=1.0).text("Mix"));
        }
    });
    Ok(())
}
