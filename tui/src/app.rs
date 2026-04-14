use rhuidean_studio_core::scale::ScaleType;
use rhuidean_studio_core::simulation::{OrbitalSystem, VelocityMode};
use rhuidean_studio_core::tuning::TuningSystem;

use crate::audio::{AudioParams, Waveform};
use crate::ui::{self, Control};

pub struct AppState {
    pub system: OrbitalSystem,
    pub running: bool,
    pub show_controls: bool,
    pub cursor: usize,

    pub ratio_p: u32,
    pub ratio_q: u32,
    pub num_orbits: usize,
    pub velocity_mode: VelocityMode,
    pub tuning: TuningSystem,
    pub scale_enabled: bool,
    pub scale_root_name: String,
    pub scale_root_hz: f64,
    pub scale_type: ScaleType,
    pub waveform: Waveform,
    pub subdivisions: u32,
    pub speed: f64,
    pub base_freq: f64,
    pub filter_enabled: bool,
    pub filter_cutoff: f64,
    pub filter_resonance: f64,
    pub delay_wet: f64,
    pub delay_time: f64,
    pub delay_feedback: f64,
    pub stereo_enabled: bool,
    pub detune: f64,
    pub chord_enabled: bool,
    pub convergence_lines: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let system = OrbitalSystem::new(3, 2, 12, VelocityMode::Linear);
        Self {
            system,
            running: false,
            show_controls: true,
            cursor: 0,

            ratio_p: 3,
            ratio_q: 2,
            num_orbits: 12,
            velocity_mode: VelocityMode::Linear,
            tuning: TuningSystem::Overtone,
            scale_enabled: false,
            scale_root_name: "A".to_string(),
            scale_root_hz: 440.0,
            scale_type: ScaleType::Ionian,
            waveform: Waveform::Sine,
            subdivisions: 1,
            speed: 1.0,
            base_freq: 220.0,
            filter_enabled: false,
            filter_cutoff: 4000.0,
            filter_resonance: 2.0,
            delay_wet: 0.0,
            delay_time: 0.3,
            delay_feedback: 0.4,
            stereo_enabled: false,
            detune: 0.0,
            chord_enabled: false,
            convergence_lines: false,
        }
    }
}

impl AppState {
    pub fn rebuild(&mut self) {
        self.system.ratio_p = self.ratio_p;
        self.system.ratio_q = self.ratio_q;
        self.system.velocity_mode = self.velocity_mode;
        self.system.subdivisions = self.subdivisions;
        self.system.rebuild_orbits(self.num_orbits);
        self.system.reset();
        self.scale_root_hz = rhuidean_studio_core::scale::note_to_hz(&self.scale_root_name);
    }

    pub fn audio_params(&self) -> AudioParams {
        AudioParams {
            waveform: self.waveform,
            filter_enabled: self.filter_enabled,
            filter_cutoff: self.filter_cutoff,
            filter_resonance: self.filter_resonance,
            delay_wet: self.delay_wet,
            delay_time: self.delay_time,
            delay_feedback: self.delay_feedback,
            stereo_enabled: self.stereo_enabled,
            detune_amount: self.detune,
            chord_enabled: self.chord_enabled,
            chord_ratio_p: self.ratio_p,
            chord_ratio_q: self.ratio_q,
        }
    }

    pub fn defaults(&mut self) {
        *self = Self::default();
    }

    pub fn current_control(&self) -> Option<Control> {
        let controls = ui::visible_controls(self);
        controls.get(self.cursor).copied()
    }

    pub fn adjust(&mut self, delta: i32) {
        let ctrl = match self.current_control() {
            Some(c) => c,
            None => return,
        };
        match ctrl {
            Control::Ratio => {
                cycle_ratio(&mut self.ratio_p, &mut self.ratio_q, delta);
                self.rebuild();
            }
            Control::Orbits => {
                self.num_orbits = (self.num_orbits as i32 + delta).clamp(2, 32) as usize;
                self.rebuild();
            }
            Control::Velocity => {
                cycle_enum(&mut self.velocity_mode, VELOCITY_MODES, delta);
                self.rebuild();
            }
            Control::Tuning => {
                cycle_enum(&mut self.tuning, TUNING_SYSTEMS, delta);
                self.rebuild();
            }
            Control::Scale => {
                self.scale_enabled = !self.scale_enabled;
            }
            Control::ScaleRoot => {
                let roots = [
                    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
                ];
                cycle_str(&mut self.scale_root_name, &roots, delta);
                self.rebuild();
            }
            Control::ScaleType => {
                cycle_enum(&mut self.scale_type, SCALE_TYPES, delta);
                self.rebuild();
            }
            Control::Waveform => {
                cycle_enum(&mut self.waveform, WAVEFORMS, delta);
            }
            Control::Subdivisions => {
                self.subdivisions = (self.subdivisions as i32 + delta).clamp(1, 12) as u32;
                self.system.subdivisions = self.subdivisions;
            }
            Control::Speed => {
                self.speed = (self.speed + delta as f64 * 0.1).clamp(-5.0, 5.0);
                self.speed = (self.speed * 10.0).round() / 10.0;
            }
            Control::BaseFreq => {
                self.base_freq = (self.base_freq + delta as f64 * 10.0).clamp(55.0, 880.0);
            }
            Control::Filter => {
                self.filter_enabled = !self.filter_enabled;
            }
            Control::Cutoff => {
                self.filter_cutoff =
                    (self.filter_cutoff + delta as f64 * 200.0).clamp(200.0, 12000.0);
            }
            Control::Resonance => {
                self.filter_resonance =
                    (self.filter_resonance + delta as f64 * 0.5).clamp(0.1, 20.0);
                self.filter_resonance = (self.filter_resonance * 10.0).round() / 10.0;
            }
            Control::DelayWet => {
                self.delay_wet = (self.delay_wet + delta as f64 * 0.05).clamp(0.0, 1.0);
                self.delay_wet = (self.delay_wet * 100.0).round() / 100.0;
            }
            Control::DelayTime => {
                self.delay_time = (self.delay_time + delta as f64 * 0.05).clamp(0.05, 1.5);
                self.delay_time = (self.delay_time * 100.0).round() / 100.0;
            }
            Control::DelayFeedback => {
                self.delay_feedback = (self.delay_feedback + delta as f64 * 0.05).clamp(0.0, 0.95);
                self.delay_feedback = (self.delay_feedback * 100.0).round() / 100.0;
            }
            Control::Stereo => {
                self.stereo_enabled = !self.stereo_enabled;
            }
            Control::Detune => {
                self.detune = (self.detune + delta as f64 * 5.0).clamp(0.0, 50.0);
            }
            Control::Chord => {
                self.chord_enabled = !self.chord_enabled;
            }
            Control::ConvergenceLines => {
                self.convergence_lines = !self.convergence_lines;
            }
        }
    }

    pub fn share_command(&self) -> String {
        let mut args = vec!["rhuidean-studio".to_string()];
        if self.ratio_p != 3 || self.ratio_q != 2 {
            args.push(format!("--ratio {}/{}", self.ratio_p, self.ratio_q));
        }
        if self.num_orbits != 12 {
            args.push(format!("--orbits {}", self.num_orbits));
        }
        if self.velocity_mode != VelocityMode::Linear {
            args.push(format!("--velocity {}", self.velocity_mode));
        }
        if self.tuning != TuningSystem::Overtone {
            args.push(format!("--tuning {}", self.tuning));
        }
        if self.speed != 1.0 {
            args.push(format!("--speed {:.1}", self.speed));
        }
        if self.base_freq != 220.0 {
            args.push(format!("--base-freq {}", self.base_freq as u32));
        }
        if self.waveform != Waveform::Sine {
            args.push(format!("--waveform {}", self.waveform));
        }
        if self.subdivisions != 1 {
            args.push(format!("--subdivisions {}", self.subdivisions));
        }
        if self.scale_enabled {
            args.push(format!("--scale {}", self.scale_type));
            args.push(format!("--scale-root {}", self.scale_root_name));
        }
        args.join(" ")
    }
}

pub const RATIOS: &[(u32, u32)] = &[
    (9, 8),
    (8, 7),
    (7, 6),
    (13, 11),
    (6, 5),
    (11, 9),
    (5, 4),
    (9, 7),
    (4, 3),
    (11, 8),
    (7, 5),
    (3, 2),
    (11, 7),
    (8, 5),
    (13, 8),
    (5, 3),
    (7, 4),
    (11, 6),
    (15, 8),
    (2, 1),
];

const VELOCITY_MODES: &[VelocityMode] = &[
    VelocityMode::Linear,
    VelocityMode::Geometric,
    VelocityMode::InverseSquare,
    VelocityMode::HarmonicSeries,
    VelocityMode::IntegerHarmonic,
];

const TUNING_SYSTEMS: &[TuningSystem] = &[
    TuningSystem::Overtone,
    TuningSystem::EqualTemperament,
    TuningSystem::JustIntonation,
    TuningSystem::Pythagorean,
];

const SCALE_TYPES: &[ScaleType] = &[
    ScaleType::Ionian,
    ScaleType::Dorian,
    ScaleType::Phrygian,
    ScaleType::Lydian,
    ScaleType::Mixolydian,
    ScaleType::Aeolian,
    ScaleType::Locrian,
    ScaleType::PentatonicMajor,
    ScaleType::PentatonicMinor,
    ScaleType::Blues,
    ScaleType::WholeTone,
    ScaleType::HarmonicMinor,
    ScaleType::MelodicMinor,
    ScaleType::Chromatic,
];

const WAVEFORMS: &[Waveform] = &[
    Waveform::Sine,
    Waveform::Triangle,
    Waveform::Square,
    Waveform::Sawtooth,
];

fn cycle_ratio(p: &mut u32, q: &mut u32, delta: i32) {
    let idx = RATIOS.iter().position(|&(rp, rq)| rp == *p && rq == *q);
    let new_idx = match idx {
        Some(i) => (i as i32 + delta).rem_euclid(RATIOS.len() as i32) as usize,
        None => 0,
    };
    *p = RATIOS[new_idx].0;
    *q = RATIOS[new_idx].1;
}

fn cycle_enum<T: Copy + PartialEq>(current: &mut T, options: &[T], delta: i32) {
    let idx = options.iter().position(|o| *o == *current);
    let new_idx = match idx {
        Some(i) => (i as i32 + delta).rem_euclid(options.len() as i32) as usize,
        None => 0,
    };
    *current = options[new_idx];
}

fn cycle_str(current: &mut String, options: &[&str], delta: i32) {
    let idx = options.iter().position(|&s| s == current.as_str());
    let new_idx = match idx {
        Some(i) => (i as i32 + delta).rem_euclid(options.len() as i32) as usize,
        None => 0,
    };
    *current = options[new_idx].to_string();
}
