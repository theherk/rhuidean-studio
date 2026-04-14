use wasm_bindgen::prelude::*;
use web_sys::{
    AudioContext, BiquadFilterNode, BiquadFilterType, DelayNode, GainNode, OscillatorNode,
    OscillatorType, StereoPannerNode,
};

pub struct AudioEngine {
    ctx: AudioContext,
    oscillator_type: OscillatorType,
    master_gain: GainNode,
    delay_send: GainNode,
    delay_node: DelayNode,
    delay_feedback: GainNode,
    delay_wet: f64,
    filter_enabled: bool,
    filter_cutoff: f64,
    filter_resonance: f64,
    stereo_enabled: bool,
    detune_amount: f64,
    chord_enabled: bool,
    chord_ratio_p: u32,
    chord_ratio_q: u32,
}

impl AudioEngine {
    pub fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;
        let master_gain = GainNode::new(&ctx)?;
        master_gain.gain().set_value(0.3);
        master_gain.connect_with_audio_node(&ctx.destination())?;
        let delay_opts = web_sys::DelayOptions::new();
        delay_opts.set_max_delay_time(2.0);
        delay_opts.set_delay_time(0.3);
        let delay_node = DelayNode::new_with_options(&ctx, &delay_opts)?;

        let delay_feedback = GainNode::new(&ctx)?;
        delay_feedback.gain().set_value(0.4);

        let delay_send = GainNode::new(&ctx)?;
        delay_send.gain().set_value(0.0);

        delay_send.connect_with_audio_node(&delay_node)?;
        delay_node.connect_with_audio_node(&delay_feedback)?;
        delay_feedback.connect_with_audio_node(&delay_node)?;
        delay_node.connect_with_audio_node(&master_gain)?;

        Ok(Self {
            ctx,
            oscillator_type: OscillatorType::Sine,
            master_gain,
            delay_send,
            delay_node,
            delay_feedback,
            delay_wet: 0.0,
            filter_enabled: false,
            filter_cutoff: 4000.0,
            filter_resonance: 2.0,
            stereo_enabled: false,
            detune_amount: 0.0,
            chord_enabled: false,
            chord_ratio_p: 3,
            chord_ratio_q: 2,
        })
    }

    pub fn set_oscillator_type(&mut self, wave: &str) {
        self.oscillator_type = match wave {
            "square" => OscillatorType::Square,
            "triangle" => OscillatorType::Triangle,
            "sawtooth" => OscillatorType::Sawtooth,
            _ => OscillatorType::Sine,
        };
    }

    pub fn set_filter_enabled(&mut self, enabled: bool) {
        self.filter_enabled = enabled;
    }

    pub fn set_filter_cutoff(&mut self, cutoff: f64) {
        self.filter_cutoff = cutoff;
    }

    pub fn set_filter_resonance(&mut self, q: f64) {
        self.filter_resonance = q;
    }

    pub fn set_delay_wet(&mut self, wet: f64) {
        self.delay_wet = wet;
        self.delay_send.gain().set_value(wet as f32);
    }

    pub fn set_delay_time(&mut self, time: f64) {
        self.delay_node.delay_time().set_value(time as f32);
    }

    pub fn set_delay_feedback(&mut self, feedback: f64) {
        self.delay_feedback
            .gain()
            .set_value(feedback.min(0.95) as f32);
    }

    pub fn set_stereo_enabled(&mut self, enabled: bool) {
        self.stereo_enabled = enabled;
    }

    pub fn set_detune_amount(&mut self, cents: f64) {
        self.detune_amount = cents;
    }

    pub fn set_chord_enabled(&mut self, enabled: bool) {
        self.chord_enabled = enabled;
    }

    pub fn set_chord_ratio(&mut self, p: u32, q: u32) {
        self.chord_ratio_p = p;
        self.chord_ratio_q = q;
    }

    pub fn play_tone(
        &self,
        frequency: f64,
        orbit_index: usize,
        num_orbits: usize,
        is_convergence: bool,
    ) -> Result<(), JsValue> {
        let gain_scale = 1.0 / (num_orbits as f64).sqrt();
        self.play_single_tone(
            frequency,
            orbit_index,
            num_orbits,
            is_convergence,
            gain_scale,
            0.0,
        )?;

        if self.chord_enabled {
            let interval = self.chord_ratio_p as f64 / self.chord_ratio_q as f64;
            self.play_single_tone(
                frequency * interval,
                orbit_index,
                num_orbits,
                is_convergence,
                gain_scale * 0.6,
                0.0,
            )?;
        }

        Ok(())
    }

    fn play_single_tone(
        &self,
        frequency: f64,
        orbit_index: usize,
        num_orbits: usize,
        is_convergence: bool,
        gain_scale: f64,
        _detune_override: f64,
    ) -> Result<(), JsValue> {
        let now = self.ctx.current_time();
        let osc = OscillatorNode::new(&self.ctx)?;
        osc.set_type(self.oscillator_type);
        osc.frequency().set_value(frequency as f32);

        if self.detune_amount > 0.0 {
            let seed = orbit_index as f64 * 7.0 + 3.0;
            let detune = ((seed * 13.37).sin()) * self.detune_amount;
            osc.detune().set_value(detune as f32);
        }

        let gain = GainNode::new(&self.ctx)?;
        gain.gain().set_value(0.0);

        let attack = 0.005;
        let base_peak = if is_convergence { 0.4 } else { 0.2 };
        let peak = base_peak * gain_scale;
        let decay = if is_convergence { 0.8 } else { 0.15 };

        let mut last_node: &web_sys::AudioNode = osc.as_ref();
        let _filter_node;

        if self.filter_enabled {
            let filter = BiquadFilterNode::new(&self.ctx)?;
            filter.set_type(BiquadFilterType::Lowpass);
            filter.q().set_value(self.filter_resonance as f32);

            let sweep_start = (self.filter_cutoff * 4.0).min(20000.0);
            filter.frequency().set_value(sweep_start as f32);
            filter.frequency().exponential_ramp_to_value_at_time(
                self.filter_cutoff as f32,
                now + attack + decay,
            )?;

            osc.connect_with_audio_node(&filter)?;
            _filter_node = filter;
            last_node = _filter_node.as_ref();
        }

        let _panner_node;
        if self.stereo_enabled && num_orbits > 1 {
            let panner = StereoPannerNode::new(&self.ctx)?;
            let t = orbit_index as f64 / (num_orbits - 1) as f64;
            let pan = (t * 2.0 - 1.0) * 0.8;
            panner.pan().set_value(pan as f32);
            last_node.connect_with_audio_node(&panner)?;
            _panner_node = panner;
            last_node = _panner_node.as_ref();
        }

        last_node.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.master_gain)?;
        gain.connect_with_audio_node(&self.delay_send)?;

        gain.gain()
            .linear_ramp_to_value_at_time(peak as f32, now + attack)?;
        gain.gain()
            .exponential_ramp_to_value_at_time(0.001, now + attack + decay)?;

        osc.start()?;
        osc.stop_with_when(now + attack + decay + 0.01)?;

        Ok(())
    }

    pub fn resume(&self) -> Result<(), JsValue> {
        let _ = self.ctx.resume()?;
        Ok(())
    }
}
