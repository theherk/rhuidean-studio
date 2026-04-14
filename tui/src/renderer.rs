use std::f64::consts::TAU;

use ratatui::prelude::*;

use rhuidean_studio_core::simulation::OrbitalSystem;

const CELL_ASPECT: f64 = 2.4;

pub fn orbit_color(index: usize, total: usize) -> Color {
    let hue = (index as f64 / total as f64) * 300.0;
    let (r, g, b) = hsl_to_rgb(hue, 0.65, 0.65);
    Color::Rgb(r, g, b)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub struct FlashState {
    pub timers: Vec<f64>,
    pub convergence_timer: f64,
    pub triggered_indices: Vec<usize>,
}

impl FlashState {
    pub fn new(n: usize) -> Self {
        Self {
            timers: vec![0.0; n],
            convergence_timer: 0.0,
            triggered_indices: Vec::new(),
        }
    }

    pub fn set_num_orbits(&mut self, n: usize) {
        self.timers = vec![0.0; n];
    }

    pub fn trigger_flash(&mut self, index: usize) {
        if index < self.timers.len() {
            self.timers[index] = 1.0;
        }
    }

    pub fn trigger_convergence(&mut self, indices: Vec<usize>) {
        self.convergence_timer = 1.0;
        self.triggered_indices = indices;
    }

    pub fn reset(&mut self) {
        for t in &mut self.timers {
            *t = 0.0;
        }
        self.convergence_timer = 0.0;
    }

    pub fn tick(&mut self, dt: f64) {
        if dt > 0.0 {
            for t in &mut self.timers {
                if *t > 0.0 {
                    *t = (*t - dt * 5.0).max(0.0);
                }
            }
            if self.convergence_timer > 0.0 {
                self.convergence_timer = (self.convergence_timer - dt * 2.0).max(0.0);
            }
        }
    }
}

fn plot(
    buf: &mut Buffer,
    area: Rect,
    cx: f64,
    cy: f64,
    max_radius: f64,
    radius_frac: f64,
    angle: f64,
    sym: &str,
    color: Color,
    overwrite: bool,
) {
    let r = radius_frac * max_radius;
    let px = cx + r * angle.sin() * CELL_ASPECT;
    let py = cy - r * angle.cos();
    let col = px.round() as i32 + area.x as i32;
    let row = py.round() as i32 + area.y as i32;
    if col >= area.x as i32
        && col < (area.x + area.width) as i32
        && row >= area.y as i32
        && row < (area.y + area.height) as i32
    {
        if let Some(cell) = buf.cell_mut(Position::new(col as u16, row as u16)) {
            if overwrite || cell.symbol() == " " {
                cell.set_symbol(sym);
                cell.set_fg(color);
            }
        }
    }
}

pub fn draw_visualization(
    frame: &mut Frame,
    area: Rect,
    system: &OrbitalSystem,
    flash: &FlashState,
    convergence_lines: bool,
) {
    let buf = frame.buffer_mut();
    let w = area.width as f64;
    let h = area.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_radius = (w / (2.0 * CELL_ASPECT)).min(h / 2.0) * 0.88;
    let num_orbits = system.orbits.len();

    for orbit in &system.orbits {
        let r = orbit.radius_fraction * max_radius;
        let steps = ((TAU * r * CELL_ASPECT).max(48.0)) as usize;
        for s in 0..steps {
            let angle = s as f64 * TAU / steps as f64;
            plot(
                buf,
                area,
                cx,
                cy,
                max_radius,
                orbit.radius_fraction,
                angle,
                "\u{00b7}",
                Color::Indexed(240),
                false,
            );
        }
    }

    for s in 0..system.subdivisions {
        let angle = s as f64 * TAU / system.subdivisions as f64;
        let steps = (max_radius * 2.0) as usize;
        let color = if s == 0 {
            Color::Indexed(246)
        } else {
            Color::Indexed(243)
        };
        for step in 0..=steps {
            let t = step as f64 / steps as f64;
            plot(
                buf,
                area,
                cx,
                cy,
                max_radius,
                t * 1.05,
                angle,
                "\u{00b7}",
                color,
                true,
            );
        }
    }

    if convergence_lines && flash.convergence_timer > 0.0 && flash.triggered_indices.len() > 1 {
        let indices = &flash.triggered_indices;
        let mut positions = Vec::new();
        for &idx in indices {
            if let Some(orbit) = system.orbits.get(idx) {
                let r = orbit.radius_fraction * max_radius;
                let px = cx + r * orbit.angle.sin() * CELL_ASPECT;
                let py = cy - r * orbit.angle.cos();
                positions.push((px, py, idx));
            }
        }
        for i in 0..positions.len() {
            let (x0, y0, idx) = positions[i];
            let (x1, y1, _) = positions[(i + 1) % positions.len()];
            let dist = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
            let steps = dist as usize;
            let color = orbit_color(idx, num_orbits);
            for s in 0..=steps {
                let t = s as f64 / steps.max(1) as f64;
                let px = x0 + t * (x1 - x0);
                let py = y0 + t * (y1 - y0);
                let col = px.round() as i32 + area.x as i32;
                let row = py.round() as i32 + area.y as i32;
                if col >= area.x as i32
                    && col < (area.x + area.width) as i32
                    && row >= area.y as i32
                    && row < (area.y + area.height) as i32
                {
                    if let Some(cell) = buf.cell_mut(Position::new(col as u16, row as u16)) {
                        cell.set_symbol("\u{00b7}");
                        cell.set_fg(color);
                    }
                }
            }
        }
    }

    for (i, orbit) in system.orbits.iter().enumerate() {
        let flash_val = flash.timers.get(i).copied().unwrap_or(0.0);
        let (sym, color) = if flash_val > 0.5 {
            ("\u{25cf}", Color::White)
        } else if flash_val > 0.1 {
            ("\u{25cf}", orbit_color(i, num_orbits))
        } else {
            ("\u{25cb}", orbit_color(i, num_orbits))
        };
        plot(
            buf,
            area,
            cx,
            cy,
            max_radius,
            orbit.radius_fraction,
            orbit.angle,
            sym,
            color,
            true,
        );
    }
}
