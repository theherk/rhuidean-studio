use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScaleType {
    Ionian,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
    PentatonicMajor,
    PentatonicMinor,
    Blues,
    WholeTone,
    HarmonicMinor,
    MelodicMinor,
    Chromatic,
}

impl ScaleType {
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            ScaleType::Ionian => &[0, 2, 4, 5, 7, 9, 11],
            ScaleType::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            ScaleType::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            ScaleType::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            ScaleType::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            ScaleType::Aeolian => &[0, 2, 3, 5, 7, 8, 10],
            ScaleType::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            ScaleType::PentatonicMajor => &[0, 2, 4, 7, 9],
            ScaleType::PentatonicMinor => &[0, 3, 5, 7, 10],
            ScaleType::Blues => &[0, 3, 5, 6, 7, 10],
            ScaleType::WholeTone => &[0, 2, 4, 6, 8, 10],
            ScaleType::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            ScaleType::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            ScaleType::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        }
    }
}

impl FromStr for ScaleType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ionian" => Ok(ScaleType::Ionian),
            "dorian" => Ok(ScaleType::Dorian),
            "phrygian" => Ok(ScaleType::Phrygian),
            "lydian" => Ok(ScaleType::Lydian),
            "mixolydian" => Ok(ScaleType::Mixolydian),
            "aeolian" => Ok(ScaleType::Aeolian),
            "locrian" => Ok(ScaleType::Locrian),
            "pentatonic_major" => Ok(ScaleType::PentatonicMajor),
            "pentatonic_minor" => Ok(ScaleType::PentatonicMinor),
            "blues" => Ok(ScaleType::Blues),
            "whole_tone" => Ok(ScaleType::WholeTone),
            "harmonic_minor" => Ok(ScaleType::HarmonicMinor),
            "melodic_minor" => Ok(ScaleType::MelodicMinor),
            "chromatic" => Ok(ScaleType::Chromatic),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ScaleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ScaleType::Ionian => "ionian",
            ScaleType::Dorian => "dorian",
            ScaleType::Phrygian => "phrygian",
            ScaleType::Lydian => "lydian",
            ScaleType::Mixolydian => "mixolydian",
            ScaleType::Aeolian => "aeolian",
            ScaleType::Locrian => "locrian",
            ScaleType::PentatonicMajor => "pentatonic_major",
            ScaleType::PentatonicMinor => "pentatonic_minor",
            ScaleType::Blues => "blues",
            ScaleType::WholeTone => "whole_tone",
            ScaleType::HarmonicMinor => "harmonic_minor",
            ScaleType::MelodicMinor => "melodic_minor",
            ScaleType::Chromatic => "chromatic",
        };
        write!(f, "{s}")
    }
}

pub fn note_to_hz(note: &str) -> f64 {
    match note {
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
    }
}

pub fn degree_frequency(orbit_index: usize, root_hz: f64, scale: &ScaleType) -> f64 {
    let intervals = scale.intervals();
    let len = intervals.len();
    let octave = orbit_index / len;
    let degree = orbit_index % len;
    root_hz * 2.0_f64.powf((octave as f64 * 12.0 + intervals[degree] as f64) / 12.0)
}
