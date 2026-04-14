use std::f64::consts::TAU;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Waveform {
    Sine,
    Triangle,
    Square,
    Sawtooth,
}

impl std::str::FromStr for Waveform {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sine" => Ok(Self::Sine),
            "triangle" => Ok(Self::Triangle),
            "square" => Ok(Self::Square),
            "sawtooth" => Ok(Self::Sawtooth),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Waveform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sine => write!(f, "sine"),
            Self::Triangle => write!(f, "triangle"),
            Self::Square => write!(f, "square"),
            Self::Sawtooth => write!(f, "sawtooth"),
        }
    }
}

struct Voice {
    frequency: f64,
    phase: f64,
    envelope: f64,
    decay_rate: f64,
    pan: f64,
    detune_hz: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

pub struct AudioParams {
    pub waveform: Waveform,
    pub filter_enabled: bool,
    pub filter_cutoff: f64,
    pub filter_resonance: f64,
    pub delay_wet: f64,
    pub delay_time: f64,
    pub delay_feedback: f64,
    pub stereo_enabled: bool,
    pub detune_amount: f64,
    pub chord_enabled: bool,
    pub chord_ratio_p: u32,
    pub chord_ratio_q: u32,
}

impl Default for AudioParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            filter_enabled: false,
            filter_cutoff: 4000.0,
            filter_resonance: 2.0,
            delay_wet: 0.0,
            delay_time: 0.3,
            delay_feedback: 0.4,
            stereo_enabled: false,
            detune_amount: 0.0,
            chord_enabled: false,
            chord_ratio_p: 3,
            chord_ratio_q: 2,
        }
    }
}

pub struct TriggerMsg {
    pub frequency: f64,
    pub orbit_index: usize,
    pub num_orbits: usize,
    pub is_convergence: bool,
}

// Known limitation: the audio render callback locks multiple Arc<Mutex<_>>
// on every sample frame. This risks priority inversion and potential glitches
// under heavy load. A lock-free ring buffer would be more robust but is a
// significant refactor. Acceptable for a TUI visualizer at typical loads.
pub fn spawn_audio_thread(
    rx: mpsc::Receiver<TriggerMsg>,
    params: Arc<Mutex<AudioParams>>,
) -> Option<cpal::Stream> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;
    let sample_rate = config.sample_rate().0 as f64;
    let channels = config.channels() as usize;

    let voices: Arc<Mutex<Vec<Voice>>> = Arc::new(Mutex::new(Vec::new()));
    let voices_trigger = voices.clone();

    let delay_len = (sample_rate * 2.0) as usize;
    let delay_buf_l: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(vec![0.0; delay_len]));
    let delay_buf_r: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(vec![0.0; delay_len]));
    let delay_pos: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    let params_render = params.clone();
    let delay_buf_l_render = delay_buf_l.clone();
    let delay_buf_r_render = delay_buf_r.clone();
    let delay_pos_render = delay_pos.clone();
    let voices_render = voices.clone();

    std::thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            let p = params.lock().unwrap();
            let gain_scale = 1.0 / (msg.num_orbits as f64).sqrt();
            let decay = if msg.is_convergence { 0.8 } else { 0.15 };
            let peak = if msg.is_convergence { 0.4 } else { 0.2 } * gain_scale;
            let pan = if p.stereo_enabled && msg.num_orbits > 1 {
                let t = msg.orbit_index as f64 / (msg.num_orbits - 1) as f64;
                (t * 2.0 - 1.0) * 0.8
            } else {
                0.0
            };
            let detune_hz = if p.detune_amount > 0.0 {
                let seed = msg.orbit_index as f64 * 7.0 + 3.0;
                (seed * 13.37).sin() * p.detune_amount * msg.frequency / 1200.0
            } else {
                0.0
            };

            let mut voices = voices_trigger.lock().unwrap();
            voices.push(Voice {
                frequency: msg.frequency,
                phase: 0.0,
                envelope: peak,
                decay_rate: (-6.0 / (decay * sample_rate)).exp(),
                pan,
                detune_hz,
                x1: 0.0,
                x2: 0.0,
                y1: 0.0,
                y2: 0.0,
            });

            if p.chord_enabled {
                let interval = p.chord_ratio_p as f64 / p.chord_ratio_q as f64;
                voices.push(Voice {
                    frequency: msg.frequency * interval,
                    phase: 0.0,
                    envelope: peak * 0.6,
                    decay_rate: (-6.0 / (decay * sample_rate)).exp(),
                    pan,
                    detune_hz,
                    x1: 0.0,
                    x2: 0.0,
                    y1: 0.0,
                    y2: 0.0,
                });
            }
        }
    });

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let p = params_render.lock().unwrap();
                let mut voices = voices_render.lock().unwrap();
                let mut delay_l = delay_buf_l_render.lock().unwrap();
                let mut delay_r = delay_buf_r_render.lock().unwrap();
                let mut dpos = delay_pos_render.lock().unwrap();

                let delay_samples = (p.delay_time * sample_rate) as usize;

                for frame in data.chunks_mut(channels) {
                    let mut mix_l = 0.0_f64;
                    let mut mix_r = 0.0_f64;

                    for voice in voices.iter_mut() {
                        let freq = voice.frequency + voice.detune_hz;
                        voice.phase += freq * TAU / sample_rate;
                        if voice.phase > TAU {
                            voice.phase -= TAU;
                        }

                        let raw = match p.waveform {
                            Waveform::Sine => voice.phase.sin(),
                            Waveform::Triangle => {
                                (2.0 * (voice.phase / TAU) - 1.0).abs() * 2.0 - 1.0
                            }
                            Waveform::Square => {
                                if voice.phase < std::f64::consts::PI {
                                    1.0
                                } else {
                                    -1.0
                                }
                            }
                            Waveform::Sawtooth => 2.0 * (voice.phase / TAU) - 1.0,
                        };

                        let sample = if p.filter_enabled {
                            let f0 = p.filter_cutoff / sample_rate;
                            let q = p.filter_resonance.max(0.5);
                            let w0 = TAU * f0;
                            let alpha = w0.sin() / (2.0 * q);
                            let cos_w0 = w0.cos();
                            let a0 = 1.0 + alpha;
                            let b0 = ((1.0 - cos_w0) / 2.0) / a0;
                            let b1 = (1.0 - cos_w0) / a0;
                            let b2 = b0;
                            let a1 = (-2.0 * cos_w0) / a0;
                            let a2 = (1.0 - alpha) / a0;
                            let y = b0 * raw + b1 * voice.x1 + b2 * voice.x2
                                - a1 * voice.y1
                                - a2 * voice.y2;
                            voice.x2 = voice.x1;
                            voice.x1 = raw;
                            voice.y2 = voice.y1;
                            voice.y1 = y;
                            y.clamp(-1.0, 1.0)
                        } else {
                            raw
                        };

                        let out = sample * voice.envelope;
                        voice.envelope *= voice.decay_rate;

                        let l_gain = ((1.0 - voice.pan) / 2.0).sqrt();
                        let r_gain = ((1.0 + voice.pan) / 2.0).sqrt();
                        mix_l += out * l_gain;
                        mix_r += out * r_gain;
                    }

                    if p.delay_wet > 0.0 && delay_samples > 0 {
                        let read_pos = (*dpos + delay_l.len() - delay_samples) % delay_l.len();
                        let dl = delay_l[read_pos];
                        let dr = delay_r[read_pos];
                        let wet = p.delay_wet;
                        let fb = p.delay_feedback.min(0.95);
                        delay_l[*dpos] = mix_l + dl * fb;
                        delay_r[*dpos] = mix_r + dr * fb;
                        mix_l += dl * wet;
                        mix_r += dr * wet;
                        *dpos = (*dpos + 1) % delay_l.len();
                    }

                    let master = 0.3;
                    if channels >= 2 {
                        frame[0] = (mix_l * master) as f32;
                        frame[1] = (mix_r * master) as f32;
                    } else {
                        frame[0] = ((mix_l + mix_r) * 0.5 * master) as f32;
                    }
                }

                voices.retain(|v| v.envelope > 0.001);
            },
            |err| eprintln!("audio error: {err}"),
            None,
        )
        .ok()?;

    stream.play().ok()?;
    Some(stream)
}
