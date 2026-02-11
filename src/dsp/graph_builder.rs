use bevy::prelude::*;
use fundsp::prelude32::*;

use crate::components::effect::{Delay, Distortion, Reverb};
use crate::components::filter::{BandPass, HighPass, LowPass};
use crate::components::synth::{Amplitude, Frequency, OscillatorType};
use crate::dsp::param::ParamHandle;

/// Holds all parameter handles for a synth entity's DSP graph.
#[derive(Component)]
pub struct SynthParams {
    pub frequency: ParamHandle,
    pub amplitude: ParamHandle,
    pub filter_cutoff: Option<ParamHandle>,
    pub filter_resonance: Option<ParamHandle>,
}

/// Build a FunDSP graph from synth component data.
///
/// Returns (graph, params) where graph is stereo out and params
/// contains all live-tweakable parameter handles.
pub fn build_synth_graph(
    osc_type: &OscillatorType,
    freq: &Frequency,
    amp: &Amplitude,
    low_pass: Option<&LowPass>,
    high_pass: Option<&HighPass>,
    band_pass: Option<&BandPass>,
    reverb_cfg: Option<&Reverb>,
    _delay: Option<&Delay>,
    distortion: Option<&Distortion>,
) -> (Box<dyn AudioUnit>, SynthParams) {
    let freq_param = ParamHandle::new("frequency", freq.0, 20.0, 20000.0);
    let amp_param = ParamHandle::new("amplitude", amp.0, 0.0, 1.0);

    let freq_s = freq_param.shared().clone();
    let amp_s = amp_param.shared().clone();

    let mut filter_cutoff_param = None;
    let mut filter_resonance_param = None;

    // Use a Net to dynamically wire the graph.
    let mut net = Net::new(0, 2);

    // Build oscillator driven by frequency parameter.
    let osc_id = match osc_type {
        OscillatorType::Sine => net.push(Box::new(var(&freq_s) >> sine())),
        OscillatorType::Saw => net.push(Box::new(var(&freq_s) >> saw())),
        OscillatorType::Square => net.push(Box::new(var(&freq_s) >> square())),
        OscillatorType::Triangle => net.push(Box::new(var(&freq_s) >> triangle())),
        OscillatorType::Noise => net.push(Box::new(noise())),
    };

    let mut last_id = osc_id;

    // Apply filter if present (priority: low-pass > high-pass > band-pass).
    if let Some(lp) = low_pass {
        let cutoff = ParamHandle::new("filter_cutoff", lp.cutoff_hz, 20.0, 20000.0);
        let res = ParamHandle::new("filter_resonance", lp.resonance, 0.1, 10.0);
        let cutoff_s = cutoff.shared().clone();
        let res_s = res.shared().clone();
        let cutoff_id = net.push(Box::new(var(&cutoff_s)));
        let res_id = net.push(Box::new(var(&res_s)));
        let filter_id = net.push(Box::new(moog()));
        net.connect(last_id, 0, filter_id, 0);
        net.connect(cutoff_id, 0, filter_id, 1);
        net.connect(res_id, 0, filter_id, 2);
        filter_cutoff_param = Some(cutoff);
        filter_resonance_param = Some(res);
        last_id = filter_id;
    } else if let Some(hp) = high_pass {
        let cutoff = ParamHandle::new("filter_cutoff", hp.cutoff_hz, 20.0, 20000.0);
        let filter_id = net.push(Box::new(highpole_hz(hp.cutoff_hz)));
        net.connect(last_id, 0, filter_id, 0);
        filter_cutoff_param = Some(cutoff);
        last_id = filter_id;
    } else if let Some(bp) = band_pass {
        let cutoff = ParamHandle::new("filter_cutoff", bp.center_hz, 20.0, 20000.0);
        let bw = ParamHandle::new("filter_resonance", bp.bandwidth, 10.0, 5000.0);
        let cutoff_s = cutoff.shared().clone();
        let bw_s = bw.shared().clone();
        let cutoff_id = net.push(Box::new(var(&cutoff_s)));
        let bw_id = net.push(Box::new(var(&bw_s)));
        let filter_id = net.push(Box::new(bandpass()));
        net.connect(last_id, 0, filter_id, 0);
        net.connect(cutoff_id, 0, filter_id, 1);
        net.connect(bw_id, 0, filter_id, 2);
        filter_cutoff_param = Some(cutoff);
        filter_resonance_param = Some(bw);
        last_id = filter_id;
    }

    // Apply distortion if present.
    if let Some(dist) = distortion {
        let drive = dist.drive;
        let mix = dist.mix;
        let dist_id = net.push(Box::new(map(move |frame: &Frame<f32, U1>| -> f32 {
            let x = frame[0];
            let saturated = (x * drive).tanh();
            x * (1.0 - mix) + saturated * mix
        })));
        net.connect(last_id, 0, dist_id, 0);
        last_id = dist_id;
    }

    // Apply amplitude via a 2-input multiply map node.
    let amp_id = net.push(Box::new(var(&amp_s)));
    let amp_mul_id = net.push(Box::new(map(|frame: &Frame<f32, U2>| -> f32 {
        frame[0] * frame[1]
    })));
    net.connect(last_id, 0, amp_mul_id, 0);
    net.connect(amp_id, 0, amp_mul_id, 1);
    last_id = amp_mul_id;

    // Split to stereo.
    let split_id = net.push(Box::new(split::<U2>()));
    net.connect(last_id, 0, split_id, 0);

    // Connect to output.
    net.connect_output(split_id, 0, 0);
    net.connect_output(split_id, 1, 1);

    // Apply reverb if present.
    let final_graph: Box<dyn AudioUnit> = if let Some(rev) = reverb_cfg {
        let room = rev.room_size;
        let time = rev.decay_time;
        let damp = rev.damping;
        let reverb_node = reverb2_stereo(room, time, damp, 1.0, lowpole_hz(6000.0));
        Box::new(net >> reverb_node)
    } else {
        Box::new(net)
    };

    let params = SynthParams {
        frequency: freq_param,
        amplitude: amp_param,
        filter_cutoff: filter_cutoff_param,
        filter_resonance: filter_resonance_param,
    };

    (final_graph, params)
}
