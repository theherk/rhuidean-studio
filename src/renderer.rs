use std::collections::VecDeque;
use std::f64::consts::TAU;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::simulation::OrbitalSystem;

const TRAIL_LENGTH: usize = 8;

#[derive(Clone, Copy)]
pub struct Theme {
    pub background: &'static str,
    pub track_alpha: f64,
    pub reference_alpha: f64,
    pub flash_color: &'static str,
    pub hue_offset: f64,
    pub hue_range: f64,
    pub saturation: f64,
    pub lightness: f64,
}

const THEME_DEFAULT: Theme = Theme {
    background: "rgb(10, 10, 15)",
    track_alpha: 0.08,
    reference_alpha: 0.25,
    flash_color: "rgba(255, 255, 200, 0.9)",
    hue_offset: 0.0,
    hue_range: 300.0,
    saturation: 70.0,
    lightness: 60.0,
};

const THEME_WARM: Theme = Theme {
    background: "rgb(20, 10, 5)",
    track_alpha: 0.1,
    reference_alpha: 0.3,
    flash_color: "rgba(255, 220, 150, 0.9)",
    hue_offset: 0.0,
    hue_range: 60.0,
    saturation: 80.0,
    lightness: 55.0,
};

const THEME_COOL: Theme = Theme {
    background: "rgb(5, 10, 20)",
    track_alpha: 0.1,
    reference_alpha: 0.3,
    flash_color: "rgba(200, 230, 255, 0.9)",
    hue_offset: 180.0,
    hue_range: 120.0,
    saturation: 65.0,
    lightness: 60.0,
};

const THEME_MONO: Theme = Theme {
    background: "rgb(10, 10, 10)",
    track_alpha: 0.1,
    reference_alpha: 0.3,
    flash_color: "rgba(255, 255, 255, 0.9)",
    hue_offset: 0.0,
    hue_range: 0.0,
    saturation: 0.0,
    lightness: 70.0,
};

const THEME_HIGH_CONTRAST: Theme = Theme {
    background: "rgb(0, 0, 0)",
    track_alpha: 0.15,
    reference_alpha: 0.5,
    flash_color: "rgba(255, 255, 100, 1.0)",
    hue_offset: 0.0,
    hue_range: 300.0,
    saturation: 100.0,
    lightness: 50.0,
};

const THEME_DEFAULT_LIGHT: Theme = Theme {
    background: "rgb(240, 240, 245)",
    track_alpha: 0.12,
    reference_alpha: 0.3,
    flash_color: "rgba(180, 150, 50, 0.9)",
    hue_offset: 0.0,
    hue_range: 300.0,
    saturation: 60.0,
    lightness: 45.0,
};

const THEME_WARM_LIGHT: Theme = Theme {
    background: "rgb(250, 245, 235)",
    track_alpha: 0.15,
    reference_alpha: 0.35,
    flash_color: "rgba(200, 150, 50, 0.9)",
    hue_offset: 0.0,
    hue_range: 60.0,
    saturation: 70.0,
    lightness: 40.0,
};

const THEME_COOL_LIGHT: Theme = Theme {
    background: "rgb(235, 240, 250)",
    track_alpha: 0.15,
    reference_alpha: 0.35,
    flash_color: "rgba(50, 100, 180, 0.9)",
    hue_offset: 180.0,
    hue_range: 120.0,
    saturation: 55.0,
    lightness: 42.0,
};

const THEME_MONO_LIGHT: Theme = Theme {
    background: "rgb(240, 240, 240)",
    track_alpha: 0.15,
    reference_alpha: 0.35,
    flash_color: "rgba(60, 60, 60, 0.9)",
    hue_offset: 0.0,
    hue_range: 0.0,
    saturation: 0.0,
    lightness: 35.0,
};

const THEME_HIGH_CONTRAST_LIGHT: Theme = Theme {
    background: "rgb(255, 255, 255)",
    track_alpha: 0.2,
    reference_alpha: 0.5,
    flash_color: "rgba(200, 150, 0, 1.0)",
    hue_offset: 0.0,
    hue_range: 300.0,
    saturation: 100.0,
    lightness: 40.0,
};

pub struct Renderer {
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
    flash_timers: Vec<f64>,
    trails: Vec<VecDeque<(f64, f64)>>,
    theme: Theme,
    theme_name: String,
    light_mode: bool,
    convergence_lines: bool,
    convergence_timer: f64,
    triggered_indices: Vec<usize>,
    spiral_trails: bool,
    spiral_blend: f64,
    spiral_mode: SpiralMode,
    spiral_points: Vec<VecDeque<(f64, f64)>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpiralMode {
    Epitrochoid,
    Adjacent,
    Star,
    Lissajous,
}

fn orbit_color(theme: &Theme, index: usize, total: usize) -> String {
    let hue = theme.hue_offset + (index as f64 / total as f64) * theme.hue_range;
    format!("hsl({hue}, {}%, {}%)", theme.saturation, theme.lightness)
}

fn orbit_color_alpha(theme: &Theme, index: usize, total: usize, alpha: f64) -> String {
    let hue = theme.hue_offset + (index as f64 / total as f64) * theme.hue_range;
    format!(
        "hsla({hue}, {}%, {}%, {alpha})",
        theme.saturation, theme.lightness
    )
}

impl Renderer {
    pub fn new(ctx: CanvasRenderingContext2d, width: f64, height: f64, num_orbits: usize) -> Self {
        Self {
            ctx,
            width,
            height,
            flash_timers: vec![0.0; num_orbits],
            trails: (0..num_orbits)
                .map(|_| VecDeque::with_capacity(TRAIL_LENGTH + 1))
                .collect(),
            theme: THEME_DEFAULT,
            theme_name: "default".to_string(),
            light_mode: false,
            convergence_lines: false,
            convergence_timer: 0.0,
            triggered_indices: Vec::new(),
            spiral_trails: false,
            spiral_blend: 0.5,
            spiral_mode: SpiralMode::Epitrochoid,
            spiral_points: (0..num_orbits).map(|_| VecDeque::new()).collect(),
        }
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }

    pub fn set_num_orbits(&mut self, n: usize) {
        self.flash_timers = vec![0.0; n];
        self.trails = (0..n)
            .map(|_| VecDeque::with_capacity(TRAIL_LENGTH + 1))
            .collect();
        self.spiral_points = (0..n).map(|_| VecDeque::new()).collect();
    }

    pub fn set_convergence_lines(&mut self, enabled: bool) {
        self.convergence_lines = enabled;
    }

    pub fn set_spiral_trails(&mut self, enabled: bool) {
        self.spiral_trails = enabled;
        if !enabled {
            for points in &mut self.spiral_points {
                points.clear();
            }
        }
    }

    pub fn set_spiral_blend(&mut self, blend: f64) {
        self.spiral_blend = blend.clamp(0.0, 1.0);
    }

    pub fn set_spiral_mode(&mut self, mode: &str) {
        self.spiral_mode = match mode {
            "adjacent" => SpiralMode::Adjacent,
            "star" => SpiralMode::Star,
            "lissajous" => SpiralMode::Lissajous,
            _ => SpiralMode::Epitrochoid,
        };
    }

    pub fn set_theme(&mut self, theme: &str) {
        self.theme_name = theme.to_string();
        self.apply_theme();
    }

    pub fn set_light_mode(&mut self, light: bool) {
        self.light_mode = light;
        self.apply_theme();
    }

    fn apply_theme(&mut self) {
        self.theme = if self.light_mode {
            match self.theme_name.as_str() {
                "warm" => THEME_WARM_LIGHT,
                "cool" => THEME_COOL_LIGHT,
                "mono" => THEME_MONO_LIGHT,
                "high_contrast" => THEME_HIGH_CONTRAST_LIGHT,
                _ => THEME_DEFAULT_LIGHT,
            }
        } else {
            match self.theme_name.as_str() {
                "warm" => THEME_WARM,
                "cool" => THEME_COOL,
                "mono" => THEME_MONO,
                "high_contrast" => THEME_HIGH_CONTRAST,
                _ => THEME_DEFAULT,
            }
        };
    }

    pub fn trigger_flash(&mut self, orbit_index: usize) {
        if orbit_index < self.flash_timers.len() {
            self.flash_timers[orbit_index] = 1.0;
        }
    }

    pub fn trigger_convergence(&mut self, indices: Vec<usize>) {
        self.convergence_timer = 1.0;
        self.triggered_indices = indices;
    }

    pub fn reset_trails(&mut self) {
        for points in &mut self.spiral_points {
            points.clear();
        }
        for trail in &mut self.trails {
            trail.clear();
        }
    }

    pub fn draw(&mut self, system: &OrbitalSystem, dt: f64) -> Result<(), JsValue> {
        let cx = self.width / 2.0;
        let cy = self.height / 2.0;
        let max_radius = self.width.min(self.height) / 2.0 * 0.85;

        self.ctx.clear_rect(0.0, 0.0, self.width, self.height);
        self.ctx.set_fill_style_str(self.theme.background);
        self.ctx.fill_rect(0.0, 0.0, self.width, self.height);

        let line_base = if self.light_mode {
            "0, 0, 0"
        } else {
            "255, 255, 255"
        };
        let flash_base = if self.light_mode {
            "180, 150, 50"
        } else {
            "255, 255, 200"
        };

        let track_color = format!("rgba({}, {})", line_base, self.theme.track_alpha);
        for orbit in &system.orbits {
            let r = orbit.radius_fraction * max_radius;
            self.ctx.begin_path();
            self.ctx.arc(cx, cy, r, 0.0, TAU)?;
            self.ctx.set_stroke_style_str(&track_color);
            self.ctx.set_line_width(1.0);
            self.ctx.stroke();
        }

        let ref_color = format!("rgba({}, {})", line_base, self.theme.reference_alpha);
        self.ctx.begin_path();
        self.ctx.move_to(cx, cy);
        self.ctx.line_to(cx, cy - max_radius - 10.0);
        self.ctx.set_stroke_style_str(&ref_color);
        self.ctx.set_line_width(2.0);
        self.ctx.stroke();

        let sub_alpha = self.theme.reference_alpha * 0.5;
        let sub_color = format!("rgba({}, {sub_alpha})", line_base);
        for s in 1..system.subdivisions {
            let angle = s as f64 * TAU / system.subdivisions as f64;
            let ex = cx + (max_radius + 10.0) * angle.sin();
            let ey = cy - (max_radius + 10.0) * angle.cos();
            self.ctx.begin_path();
            self.ctx.move_to(cx, cy);
            self.ctx.line_to(ex, ey);
            self.ctx.set_stroke_style_str(&sub_color);
            self.ctx.set_line_width(1.0);
            self.ctx.stroke();
        }

        let num_orbits = system.orbits.len();

        if self.spiral_trails && num_orbits >= 2 {
            let max_spiral = 6000;
            let b = self.spiral_blend;

            let num_traces = match self.spiral_mode {
                SpiralMode::Epitrochoid => 1,
                SpiralMode::Adjacent => num_orbits - 1,
                SpiralMode::Star => (num_orbits / 3).max(1),
                SpiralMode::Lissajous => (num_orbits / 2).max(1),
            };

            for i in 0..num_traces {
                if i >= self.spiral_points.len() {
                    break;
                }

                let (a1, a2, r1, r2) = match self.spiral_mode {
                    SpiralMode::Epitrochoid => {
                        let inner = &system.orbits[0];
                        let outer = &system.orbits[num_orbits - 1];
                        let big_r = outer.radius_fraction;
                        let small_r = inner.radius_fraction * b;
                        (inner.angle, outer.angle, big_r, small_r)
                    }
                    SpiralMode::Adjacent => {
                        let o1 = &system.orbits[i];
                        let o2 = &system.orbits[i + 1];
                        let r_big = o2.radius_fraction;
                        let r_small = (o2.radius_fraction - o1.radius_fraction) * b;
                        (o1.angle, o2.angle, r_big, r_small)
                    }
                    SpiralMode::Star => {
                        let step = num_orbits / num_traces;
                        let idx1 = 0;
                        let idx2 = ((i + 1) * step).min(num_orbits - 1);
                        let o1 = &system.orbits[idx1];
                        let o2 = &system.orbits[idx2];
                        let r_big = o2.radius_fraction;
                        let r_small = o1.radius_fraction * b;
                        (o1.angle, o2.angle, r_big, r_small)
                    }
                    SpiralMode::Lissajous => {
                        let idx1 = i;
                        let idx2 = num_orbits - 1 - i;
                        if idx1 >= idx2 {
                            continue;
                        }
                        let o1 = &system.orbits[idx1];
                        let o2 = &system.orbits[idx2];
                        let r_big = o2.radius_fraction * 0.5;
                        let r_small = o1.radius_fraction * b;
                        (o1.angle, o2.angle, r_big, r_small)
                    }
                };

                let nx = r1 * a2.sin() + r2 * a1.sin();
                let ny = -(r1 * a2.cos() + r2 * a1.cos());

                self.spiral_points[i].push_back((nx, ny));
                while self.spiral_points[i].len() > max_spiral {
                    self.spiral_points[i].pop_front();
                }
            }

            for i in 0..num_traces.min(self.spiral_points.len()) {
                let points = &self.spiral_points[i];
                let len = points.len();
                if len >= 2 {
                    self.ctx.begin_path();
                    let (nx0, ny0) = points[0];
                    self.ctx
                        .move_to(cx + nx0 * max_radius, cy + ny0 * max_radius);
                    for j in 1..len {
                        let (nxj, nyj) = points[j];
                        self.ctx
                            .line_to(cx + nxj * max_radius, cy + nyj * max_radius);
                    }
                    self.ctx.set_stroke_style_str(&orbit_color_alpha(
                        &self.theme,
                        i,
                        num_traces.max(2),
                        0.35,
                    ));
                    self.ctx.set_line_width(0.8);
                    self.ctx.stroke();
                }
            }
        }

        let mut current_positions = Vec::with_capacity(num_orbits);

        for (i, orbit) in system.orbits.iter().enumerate() {
            let r = orbit.radius_fraction * max_radius;
            let x = cx + r * orbit.angle.sin();
            let y = cy - r * orbit.angle.cos();
            current_positions.push((x, y));

            if i < self.trails.len() {
                let nx = orbit.radius_fraction * orbit.angle.sin();
                let ny = -(orbit.radius_fraction * orbit.angle.cos());
                self.trails[i].push_back((nx, ny));
                while self.trails[i].len() > TRAIL_LENGTH {
                    self.trails[i].pop_front();
                }
            }

            let trail = &self.trails[i];
            let trail_len = trail.len();
            if trail_len >= 2 {
                let segments = trail_len - 1;
                for j in 0..segments {
                    let (nx0, ny0) = trail[j];
                    let (nx1, ny1) = trail[j + 1];
                    let t = (j + 1) as f64 / segments as f64;
                    let alpha = t * 0.5;
                    let width = 2.0 + t * 4.0;
                    self.ctx.begin_path();
                    self.ctx
                        .move_to(cx + nx0 * max_radius, cy + ny0 * max_radius);
                    self.ctx
                        .line_to(cx + nx1 * max_radius, cy + ny1 * max_radius);
                    self.ctx.set_stroke_style_str(&orbit_color_alpha(
                        &self.theme,
                        i,
                        num_orbits,
                        alpha,
                    ));
                    self.ctx.set_line_width(width);
                    self.ctx.set_line_cap("round");
                    self.ctx.stroke();
                }
            }

            let flash = self.flash_timers.get(i).copied().unwrap_or(0.0);
            let circle_radius = 7.0 + flash * 5.0;

            self.ctx.begin_path();
            self.ctx.arc(x, y, circle_radius, 0.0, TAU)?;

            if flash > 0.1 {
                self.ctx.set_fill_style_str(self.theme.flash_color);
            } else {
                self.ctx
                    .set_fill_style_str(&orbit_color(&self.theme, i, num_orbits));
            }
            self.ctx.fill();

            if flash > 0.1 {
                self.ctx.begin_path();
                self.ctx.arc(x, y, circle_radius + flash * 8.0, 0.0, TAU)?;
                let alpha = flash * 0.4;
                self.ctx
                    .set_stroke_style_str(&format!("rgba({flash_base}, {alpha})"));
                self.ctx.set_line_width(2.0);
                self.ctx.stroke();
            }
        }

        if self.convergence_lines
            && self.convergence_timer > 0.0
            && self.triggered_indices.len() > 1
        {
            let alpha = self.convergence_timer * 0.6;
            let width = 1.0 + self.convergence_timer * 2.0;
            let color = format!("rgba({flash_base}, {alpha})");
            self.ctx.set_stroke_style_str(&color);
            self.ctx.set_line_width(width);

            let indices = &self.triggered_indices;
            let n = indices.len();
            for i in 0..n {
                if let (Some(&(x0, y0)), Some(&(x1, y1))) = (
                    current_positions.get(indices[i]),
                    current_positions.get(indices[(i + 1) % n]),
                ) {
                    self.ctx.begin_path();
                    self.ctx.move_to(x0, y0);
                    self.ctx.line_to(x1, y1);
                    self.ctx.stroke();
                }
            }
        }

        if dt > 0.0 {
            for timer in &mut self.flash_timers {
                if *timer > 0.0 {
                    *timer = (*timer - dt * 5.0).max(0.0);
                }
            }
            if self.convergence_timer > 0.0 {
                self.convergence_timer = (self.convergence_timer - dt * 2.0).max(0.0);
            }
        }

        Ok(())
    }
}
