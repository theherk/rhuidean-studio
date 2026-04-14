#![allow(dead_code)]

mod audio;
mod renderer;
mod scale;
mod simulation;
mod tuning;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use audio::AudioEngine;
use renderer::Renderer;
use scale::ScaleType;
use simulation::{OrbitalSystem, VelocityMode};
use tuning::TuningSystem;

#[wasm_bindgen]
pub struct RhuideanStudio {
    system: OrbitalSystem,
    renderer: Renderer,
    audio: AudioEngine,
    tuning: TuningSystem,
    base_freq: f64,
    speed: f64,
    running: bool,
    last_time: Option<f64>,
    scale_enabled: bool,
    scale_type: ScaleType,
    scale_root: f64,
}

#[wasm_bindgen]
impl RhuideanStudio {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<RhuideanStudio, JsValue> {
        let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?.unwrap().dyn_into()?;

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        let num_orbits = 12;
        let system = OrbitalSystem::new(3, 2, num_orbits, VelocityMode::Linear);
        let renderer = Renderer::new(ctx, width, height, num_orbits);
        let audio = AudioEngine::new()?;

        Ok(RhuideanStudio {
            system,
            renderer,
            audio,
            tuning: TuningSystem::Overtone,
            base_freq: 220.0,
            speed: 1.0,
            running: false,
            last_time: None,
            scale_enabled: false,
            scale_type: ScaleType::Ionian,
            scale_root: 261.63,
        })
    }

    pub fn set_ratio(&mut self, p: u32, q: u32) {
        self.system.ratio_p = p;
        self.system.ratio_q = q;
        self.audio.set_chord_ratio(p, q);
        let n = self.system.orbits.len();
        self.system.rebuild_orbits(n);
        self.system.reset();
    }

    pub fn set_num_orbits(&mut self, n: usize) {
        self.system.rebuild_orbits(n);
        self.system.reset();
        self.renderer.set_num_orbits(n);
    }

    pub fn set_velocity_mode(&mut self, mode: &str) {
        self.system.velocity_mode = match mode {
            "geometric" => VelocityMode::Geometric,
            "inverse_square" => VelocityMode::InverseSquare,
            "harmonic_series" => VelocityMode::HarmonicSeries,
            "integer_harmonic" => VelocityMode::IntegerHarmonic,
            _ => VelocityMode::Linear,
        };
        let n = self.system.orbits.len();
        self.system.rebuild_orbits(n);
        self.system.reset();
    }

    pub fn set_tuning(&mut self, tuning: &str) {
        self.tuning = match tuning {
            "equal_temperament" => TuningSystem::EqualTemperament,
            "just_intonation" => TuningSystem::JustIntonation,
            "pythagorean" => TuningSystem::Pythagorean,
            _ => TuningSystem::Overtone,
        };
    }

    pub fn set_subdivisions(&mut self, n: u32) {
        self.system.subdivisions = n.max(1);
    }

    pub fn set_waveform(&mut self, wave: &str) {
        self.audio.set_oscillator_type(wave);
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn set_base_freq(&mut self, freq: f64) {
        self.base_freq = freq;
    }

    pub fn set_scale_enabled(&mut self, enabled: bool) {
        self.scale_enabled = enabled;
    }

    pub fn set_scale_type(&mut self, scale: &str) {
        self.scale_type = ScaleType::from_str(scale);
    }

    pub fn set_scale_root(&mut self, note: &str) {
        self.scale_root = match note {
            "C" => 261.63,
            "C#" => 277.18,
            "D" => 293.66,
            "D#" => 311.13,
            "E" => 329.63,
            "F" => 349.23,
            "F#" => 369.99,
            "G" => 392.00,
            "G#" => 415.30,
            "A" => 440.00,
            "A#" => 466.16,
            "B" => 493.88,
            _ => 261.63,
        };
    }

    pub fn set_filter_enabled(&mut self, enabled: bool) {
        self.audio.set_filter_enabled(enabled);
    }

    pub fn set_filter_cutoff(&mut self, cutoff: f64) {
        self.audio.set_filter_cutoff(cutoff);
    }

    pub fn set_filter_resonance(&mut self, q: f64) {
        self.audio.set_filter_resonance(q);
    }

    pub fn set_delay_wet(&mut self, wet: f64) {
        self.audio.set_delay_wet(wet);
    }

    pub fn set_delay_time(&mut self, time: f64) {
        self.audio.set_delay_time(time);
    }

    pub fn set_delay_feedback(&mut self, feedback: f64) {
        self.audio.set_delay_feedback(feedback);
    }

    pub fn set_stereo_enabled(&mut self, enabled: bool) {
        self.audio.set_stereo_enabled(enabled);
    }

    pub fn set_detune_amount(&mut self, cents: f64) {
        self.audio.set_detune_amount(cents);
    }

    pub fn set_chord_enabled(&mut self, enabled: bool) {
        self.audio.set_chord_enabled(enabled);
    }

    pub fn set_convergence_lines(&mut self, enabled: bool) {
        self.renderer.set_convergence_lines(enabled);
    }

    pub fn set_spiral_trails(&mut self, enabled: bool) {
        self.renderer.set_spiral_trails(enabled);
    }

    pub fn set_spiral_blend(&mut self, blend: f64) {
        self.renderer.set_spiral_blend(blend);
    }

    pub fn set_spiral_mode(&mut self, mode: &str) {
        self.renderer.set_spiral_mode(mode);
    }

    pub fn set_theme(&mut self, theme: &str) {
        self.renderer.set_theme(theme);
    }

    pub fn set_light_mode(&mut self, light: bool) {
        self.renderer.set_light_mode(light);
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.renderer.resize(width, height);
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        self.audio.resume()?;
        self.running = true;
        self.last_time = None;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.last_time = None;
    }

    pub fn reset(&mut self) {
        self.system.reset();
        self.renderer.reset_trails();
        self.last_time = None;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn frame(&mut self, timestamp: f64) -> Result<(), JsValue> {
        let dt = if let Some(last) = self.last_time {
            ((timestamp - last) / 1000.0).min(0.05)
        } else {
            0.0
        };
        self.last_time = Some(timestamp);

        if self.running {
            let scaled_dt = dt * self.speed;
            let events = self.system.tick(scaled_dt);

            let num_orbits = self.system.orbits.len();
            let is_convergence = events.len() == num_orbits && num_orbits > 1;

            for event in &events {
                let freq = if self.scale_enabled {
                    scale::degree_frequency(event.orbit_index, self.scale_root, &self.scale_type)
                } else {
                    self.tuning
                        .frequency(event.orbit_index, num_orbits, self.base_freq)
                };
                let _ = self
                    .audio
                    .play_tone(freq, event.orbit_index, num_orbits, is_convergence);
                self.renderer.trigger_flash(event.orbit_index);
            }

            if events.len() >= 2 {
                let indices: Vec<usize> = events.iter().map(|e| e.orbit_index).collect();
                self.renderer.trigger_convergence(indices);
            }
        }

        self.renderer.draw(&self.system, dt)?;
        Ok(())
    }

    pub fn get_intervals_json(&self) -> JsValue {
        serde_wasm_bindgen::to_value(tuning::INTERVALS).unwrap_or(JsValue::NULL)
    }
}
