use std::f64::consts::TAU;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VelocityMode {
    Linear,
    Geometric,
    InverseSquare,
    HarmonicSeries,
    IntegerHarmonic,
}

impl FromStr for VelocityMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linear" => Ok(Self::Linear),
            "geometric" => Ok(Self::Geometric),
            "inverse_square" => Ok(Self::InverseSquare),
            "harmonic_series" => Ok(Self::HarmonicSeries),
            "integer_harmonic" => Ok(Self::IntegerHarmonic),
            _ => Err(()),
        }
    }
}

impl fmt::Display for VelocityMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Linear => write!(f, "linear"),
            Self::Geometric => write!(f, "geometric"),
            Self::InverseSquare => write!(f, "inverse_square"),
            Self::HarmonicSeries => write!(f, "harmonic_series"),
            Self::IntegerHarmonic => write!(f, "integer_harmonic"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Orbit {
    pub radius_fraction: f64,
    pub angle: f64,
    pub angular_velocity: f64,
    pub prev_angle: f64,
}

impl Orbit {
    pub fn crossings(&self, subdivisions: u32) -> Vec<usize> {
        let mut crossed = Vec::new();
        for s in 0..subdivisions {
            let ref_angle = s as f64 * TAU / subdivisions as f64;
            let prev = (self.prev_angle - ref_angle).rem_euclid(TAU);
            let curr = (self.angle - ref_angle).rem_euclid(TAU);
            if (prev > TAU * 0.75 && curr < TAU * 0.25) || (prev < TAU * 0.25 && curr > TAU * 0.75)
            {
                crossed.push(s as usize);
            }
        }
        crossed
    }
}

#[derive(Clone, Debug)]
pub struct TriggerEvent {
    pub orbit_index: usize,
}

#[derive(Clone, Debug)]
pub struct OrbitalSystem {
    pub ratio_p: u32,
    pub ratio_q: u32,
    pub velocity_mode: VelocityMode,
    pub orbits: Vec<Orbit>,
    pub elapsed: f64,
    pub subdivisions: u32,
}

impl OrbitalSystem {
    pub fn new(ratio_p: u32, ratio_q: u32, num_orbits: usize, velocity_mode: VelocityMode) -> Self {
        let mut system = Self {
            ratio_p,
            ratio_q,
            velocity_mode,
            orbits: Vec::with_capacity(num_orbits),
            elapsed: 0.0,
            subdivisions: 1,
        };
        system.rebuild_orbits(num_orbits);
        system
    }

    pub fn rebuild_orbits(&mut self, num_orbits: usize) {
        self.orbits.clear();
        if num_orbits == 0 {
            return;
        }

        let base_speed = std::f64::consts::E / TAU;
        let ratio = self.ratio_p as f64 / self.ratio_q as f64;

        for i in 0..num_orbits {
            let t = if num_orbits > 1 {
                i as f64 / (num_orbits - 1) as f64
            } else {
                0.0
            };

            let radius_fraction = 0.15 + t * 0.85;

            let normalized_speed = match self.velocity_mode {
                VelocityMode::Linear => ratio + t * (1.0 - ratio),
                VelocityMode::Geometric => ratio * (1.0 / ratio).powf(t),
                VelocityMode::InverseSquare => {
                    let r_inner = 0.15_f64;
                    let r_outer = 1.0_f64;
                    let r = radius_fraction;
                    let c1 = ratio * r_inner * r_inner;
                    let c2 = 1.0 * r_outer * r_outer;
                    let c = c1 + t * (c2 - c1);
                    c / (r * r)
                }
                VelocityMode::HarmonicSeries => {
                    if num_orbits <= 1 {
                        ratio
                    } else {
                        let h_inner = 1.0;
                        let h_outer = 1.0 / num_orbits as f64;
                        let h = 1.0 / (i as f64 + 1.0);
                        let t_h = (h - h_outer) / (h_inner - h_outer);
                        1.0 + t_h * (ratio - 1.0)
                    }
                }
                VelocityMode::IntegerHarmonic => {
                    let p = self.ratio_p as f64;
                    let q = self.ratio_q as f64;
                    let v = p + t * (q - p);
                    v.round().max(1.0) / q
                }
            };

            let angular_velocity = normalized_speed * base_speed * TAU;

            self.orbits.push(Orbit {
                radius_fraction,
                angle: 0.0,
                angular_velocity,
                prev_angle: 0.0,
            });
        }
    }

    pub fn tick(&mut self, dt: f64) -> Vec<TriggerEvent> {
        self.elapsed += dt;
        let mut events = Vec::new();

        for (i, orbit) in self.orbits.iter_mut().enumerate() {
            orbit.prev_angle = orbit.angle;
            orbit.angle += orbit.angular_velocity * dt;

            let crossed = orbit.crossings(self.subdivisions);
            if !crossed.is_empty() {
                events.push(TriggerEvent { orbit_index: i });
            }

            orbit.angle = orbit.angle.rem_euclid(TAU);
            orbit.prev_angle = orbit.prev_angle.rem_euclid(TAU);
        }

        events
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        for orbit in &mut self.orbits {
            orbit.angle = 0.0;
            orbit.prev_angle = 0.0;
        }
    }
}
