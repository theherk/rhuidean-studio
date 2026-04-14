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

    pub fn from_str(s: &str) -> Self {
        match s {
            "dorian" => ScaleType::Dorian,
            "phrygian" => ScaleType::Phrygian,
            "lydian" => ScaleType::Lydian,
            "mixolydian" => ScaleType::Mixolydian,
            "aeolian" => ScaleType::Aeolian,
            "locrian" => ScaleType::Locrian,
            "pentatonic_major" => ScaleType::PentatonicMajor,
            "pentatonic_minor" => ScaleType::PentatonicMinor,
            "blues" => ScaleType::Blues,
            "whole_tone" => ScaleType::WholeTone,
            "harmonic_minor" => ScaleType::HarmonicMinor,
            "melodic_minor" => ScaleType::MelodicMinor,
            "chromatic" => ScaleType::Chromatic,
            _ => ScaleType::Ionian,
        }
    }
}

pub fn degree_frequency(orbit_index: usize, root_hz: f64, scale: &ScaleType) -> f64 {
    let intervals = scale.intervals();
    let len = intervals.len();
    let octave = orbit_index / len;
    let degree = orbit_index % len;
    root_hz * 2.0_f64.powf((octave as f64 * 12.0 + intervals[degree] as f64) / 12.0)
}
