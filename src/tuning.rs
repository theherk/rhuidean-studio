use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Interval {
    pub numerator: u32,
    pub denominator: u32,
    pub cents: f64,
    pub name: &'static str,
    pub group: &'static str,
}

pub const INTERVALS: &[Interval] = &[
    Interval {
        numerator: 9,
        denominator: 8,
        cents: 203.91,
        name: "Major second (Pythagorean)",
        group: "Seconds",
    },
    Interval {
        numerator: 8,
        denominator: 7,
        cents: 231.17,
        name: "Septimal whole tone",
        group: "Seconds",
    },
    Interval {
        numerator: 7,
        denominator: 6,
        cents: 266.87,
        name: "Septimal minor third",
        group: "Thirds",
    },
    Interval {
        numerator: 13,
        denominator: 11,
        cents: 289.21,
        name: "Tridecimal neutral third",
        group: "Thirds",
    },
    Interval {
        numerator: 6,
        denominator: 5,
        cents: 315.64,
        name: "Just minor third",
        group: "Thirds",
    },
    Interval {
        numerator: 11,
        denominator: 9,
        cents: 347.41,
        name: "Undecimal neutral third",
        group: "Thirds",
    },
    Interval {
        numerator: 5,
        denominator: 4,
        cents: 386.31,
        name: "Just major third",
        group: "Thirds",
    },
    Interval {
        numerator: 9,
        denominator: 7,
        cents: 435.08,
        name: "Septimal major third",
        group: "Thirds",
    },
    Interval {
        numerator: 4,
        denominator: 3,
        cents: 498.04,
        name: "Perfect fourth",
        group: "Fourths",
    },
    Interval {
        numerator: 11,
        denominator: 8,
        cents: 551.32,
        name: "Undecimal superfourth",
        group: "Fourths",
    },
    Interval {
        numerator: 7,
        denominator: 5,
        cents: 582.51,
        name: "Septimal tritone",
        group: "Tritones",
    },
    Interval {
        numerator: 3,
        denominator: 2,
        cents: 701.96,
        name: "Perfect fifth",
        group: "Fifths",
    },
    Interval {
        numerator: 11,
        denominator: 7,
        cents: 782.49,
        name: "Undecimal augmented fifth",
        group: "Fifths",
    },
    Interval {
        numerator: 8,
        denominator: 5,
        cents: 813.69,
        name: "Just minor sixth",
        group: "Sixths",
    },
    Interval {
        numerator: 13,
        denominator: 8,
        cents: 840.53,
        name: "Tridecimal neutral sixth",
        group: "Sixths",
    },
    Interval {
        numerator: 5,
        denominator: 3,
        cents: 884.36,
        name: "Just major sixth",
        group: "Sixths",
    },
    Interval {
        numerator: 7,
        denominator: 4,
        cents: 968.83,
        name: "Harmonic seventh",
        group: "Sevenths",
    },
    Interval {
        numerator: 11,
        denominator: 6,
        cents: 1049.36,
        name: "Undecimal neutral seventh",
        group: "Sevenths",
    },
    Interval {
        numerator: 15,
        denominator: 8,
        cents: 1088.27,
        name: "Just major seventh",
        group: "Sevenths",
    },
    Interval {
        numerator: 2,
        denominator: 1,
        cents: 1200.0,
        name: "Octave",
        group: "Octave",
    },
];

pub fn cents_from_ratio(p: f64, q: f64) -> f64 {
    1200.0 * (p / q).log2()
}

pub fn lookup_interval(numerator: u32, denominator: u32) -> Option<&'static Interval> {
    INTERVALS
        .iter()
        .find(|i| i.numerator == numerator && i.denominator == denominator)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum TuningSystem {
    Overtone,
    EqualTemperament,
    JustIntonation,
    Pythagorean,
}

const JUST_RATIOS_12: [f64; 12] = [
    1.0,
    16.0 / 15.0,
    9.0 / 8.0,
    6.0 / 5.0,
    5.0 / 4.0,
    4.0 / 3.0,
    7.0 / 5.0,
    3.0 / 2.0,
    8.0 / 5.0,
    5.0 / 3.0,
    7.0 / 4.0,
    15.0 / 8.0,
];

impl TuningSystem {
    pub fn frequency(&self, orbit_index: usize, num_orbits: usize, base_freq: f64) -> f64 {
        match self {
            TuningSystem::Overtone => base_freq * (orbit_index as f64 + 1.0),
            TuningSystem::EqualTemperament => {
                base_freq * 2.0_f64.powf(orbit_index as f64 / num_orbits as f64)
            }
            TuningSystem::JustIntonation => {
                let octave = orbit_index / JUST_RATIOS_12.len();
                let degree = orbit_index % JUST_RATIOS_12.len();
                base_freq * 2.0_f64.powi(octave as i32) * JUST_RATIOS_12[degree]
            }
            TuningSystem::Pythagorean => {
                let ratio = 3.0_f64.powi(orbit_index as i32) / 2.0_f64.powi(orbit_index as i32);
                let normalized = ratio / 2.0_f64.powf(ratio.log2().floor());
                base_freq * normalized
            }
        }
    }
}
