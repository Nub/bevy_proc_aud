pub mod components;
pub mod dsp;
pub mod plugin;
pub mod presets;
pub mod systems;

pub mod prelude {
    pub use crate::components::effect::{Delay, Distortion, Reverb};
    pub use crate::components::filter::{BandPass, HighPass, LowPass};
    pub use crate::components::synth::{Amplitude, Frequency, OscillatorType, Synth};
    pub use crate::dsp::source::ProceduralAudio;
    pub use crate::plugin::BevyProcAudPlugin;
    pub use crate::presets::blunt_impact::BluntImpact;
    pub use crate::presets::ear_ringing::EarRinging;
    pub use crate::presets::heartbeat::Heartbeat;
    pub use crate::presets::sword_slash::SwordSlash;
}
