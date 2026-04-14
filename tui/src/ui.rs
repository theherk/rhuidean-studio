use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Padding};

use crate::app::AppState;

pub const PANEL_WIDTH: u16 = 44;

#[derive(Clone, Copy, PartialEq)]
pub enum Control {
    Ratio,
    Orbits,
    Velocity,
    Tuning,
    Scale,
    ScaleRoot,
    ScaleType,
    Waveform,
    Subdivisions,
    Speed,
    BaseFreq,
    Filter,
    Cutoff,
    Resonance,
    DelayWet,
    DelayTime,
    DelayFeedback,
    Stereo,
    Detune,
    Chord,
    ConvergenceLines,
}

const CONTROLS: &[Control] = &[
    Control::Ratio,
    Control::Orbits,
    Control::Velocity,
    Control::Tuning,
    Control::Scale,
    Control::ScaleRoot,
    Control::ScaleType,
    Control::Waveform,
    Control::Subdivisions,
    Control::Speed,
    Control::BaseFreq,
    Control::Filter,
    Control::Cutoff,
    Control::Resonance,
    Control::DelayWet,
    Control::DelayTime,
    Control::DelayFeedback,
    Control::Stereo,
    Control::Detune,
    Control::Chord,
    Control::ConvergenceLines,
];

pub fn visible_controls(state: &AppState) -> Vec<Control> {
    CONTROLS
        .iter()
        .filter(|c| {
            if !state.scale_enabled && matches!(c, Control::ScaleRoot | Control::ScaleType) {
                return false;
            }
            true
        })
        .copied()
        .collect()
}

pub fn control_count(state: &AppState) -> usize {
    visible_controls(state).len()
}

const VELOCITY_OPTS: &[(&str, &str)] = &[
    ("linear", "Linear"),
    ("geometric", "Geometric"),
    ("inverse_square", "Inv.Square"),
    ("harmonic_series", "Harmonic"),
    ("integer_harmonic", "Int.Harm."),
];

const TUNING_OPTS: &[(&str, &str)] = &[
    ("overtone", "Overtone"),
    ("equal_temperament", "EqualTemp"),
    ("just_intonation", "JustInt"),
    ("pythagorean", "Pythag."),
];

const WAVEFORM_OPTS: &[(&str, &str)] = &[
    ("sine", "Sine"),
    ("triangle", "Triangle"),
    ("square", "Square"),
    ("sawtooth", "Sawtooth"),
];

const NOTE_OPTS: &[(&str, &str)] = &[
    ("C", "C"),
    ("C#", "C\u{266f}/D\u{266d}"),
    ("D", "D"),
    ("D#", "D\u{266f}/E\u{266d}"),
    ("E", "E"),
    ("F", "F"),
    ("F#", "F\u{266f}/G\u{266d}"),
    ("G", "G"),
    ("G#", "G\u{266f}/A\u{266d}"),
    ("A", "A"),
    ("A#", "A\u{266f}/B\u{266d}"),
    ("B", "B"),
];

const SCALE_OPTS: &[(&str, &str)] = &[
    ("ionian", "Ionian"),
    ("dorian", "Dorian"),
    ("phrygian", "Phrygian"),
    ("lydian", "Lydian"),
    ("mixolydian", "Mixolyd."),
    ("aeolian", "Aeolian"),
    ("locrian", "Locrian"),
    ("pentatonic_major", "Pent.Maj"),
    ("pentatonic_minor", "Pent.Min"),
    ("blues", "Blues"),
    ("whole_tone", "WholeTone"),
    ("harmonic_minor", "Harm.Min"),
    ("melodic_minor", "Mel.Min"),
    ("chromatic", "Chromatic"),
];

use crate::app::RATIOS;

fn render_cycle_selector(
    current: &str,
    options: &[(&str, &str)],
    avail_width: usize,
    selected: bool,
) -> Line<'static> {
    let idx = options.iter().position(|(k, _)| *k == current).unwrap_or(0);

    let active_style = Style::default().fg(Color::Cyan).bold();
    let neighbor_style = Style::default().fg(Color::DarkGray);
    let dot_style = Style::default().fg(Color::Gray);

    let active_text = options[idx].1;
    let active_len = active_text.chars().count();

    let budget = avail_width.saturating_sub(active_len + 4);
    let half = budget / 2;

    let mut left_items: Vec<String> = Vec::new();
    let mut left_len = 0;
    let mut li = idx;
    loop {
        if li == 0 {
            li = options.len();
        }
        li -= 1;
        let text = options[li].1;
        let tl = text.chars().count() + 1;
        if left_len + tl > half {
            break;
        }
        left_items.push(text.to_string());
        left_len += tl;
    }
    left_items.reverse();

    let mut right_items: Vec<String> = Vec::new();
    let mut right_len = 0;
    let mut ri = idx;
    loop {
        ri = (ri + 1) % options.len();
        if ri == idx {
            break;
        }
        let text = options[ri].1;
        let tl = text.chars().count() + 1;
        if right_len + tl > half {
            break;
        }
        right_items.push(text.to_string());
        right_len += tl;
    }

    let mut spans: Vec<Span<'static>> = Vec::new();

    if !left_items.is_empty() {
        spans.push(Span::styled("\u{2039} ", dot_style));
        for (i, item) in left_items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ", neighbor_style));
            }
            spans.push(Span::styled(item.clone(), neighbor_style));
        }
        spans.push(Span::raw(" "));
    } else {
        spans.push(Span::raw("  "));
    }

    spans.push(Span::styled(
        active_text.to_string(),
        if selected {
            active_style
        } else {
            Style::default().fg(Color::Cyan)
        },
    ));

    if !right_items.is_empty() {
        spans.push(Span::raw(" "));
        for (i, item) in right_items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ", neighbor_style));
            }
            spans.push(Span::styled(item.clone(), neighbor_style));
        }
        spans.push(Span::styled(" \u{203a}", dot_style));
    }

    Line::from(spans)
}

fn render_ratio_selector(p: u32, q: u32, avail_width: usize, selected: bool) -> Line<'static> {
    let idx = RATIOS
        .iter()
        .position(|&(rp, rq)| rp == p && rq == q)
        .unwrap_or(0);

    let active_style = Style::default().fg(Color::Cyan).bold();
    let neighbor_style = Style::default().fg(Color::DarkGray);
    let dot_style = Style::default().fg(Color::Gray);

    let active_text = format!("{p}/{q}");
    let active_len = active_text.chars().count();
    let budget = avail_width.saturating_sub(active_len + 4);
    let half = budget / 2;

    let mut left_items: Vec<String> = Vec::new();
    let mut left_len = 0;
    let mut li = idx;
    loop {
        if li == 0 {
            li = RATIOS.len();
        }
        li -= 1;
        let (rp, rq) = RATIOS[li];
        let text = format!("{rp}/{rq}");
        let tl = text.chars().count() + 1;
        if left_len + tl > half {
            break;
        }
        left_items.push(text);
        left_len += tl;
    }
    left_items.reverse();

    let mut right_items: Vec<String> = Vec::new();
    let mut right_len = 0;
    let mut ri = idx;
    loop {
        ri = (ri + 1) % RATIOS.len();
        if ri == idx {
            break;
        }
        let (rp, rq) = RATIOS[ri];
        let text = format!("{rp}/{rq}");
        let tl = text.chars().count() + 1;
        if right_len + tl > half {
            break;
        }
        right_items.push(text);
        right_len += tl;
    }

    let mut spans: Vec<Span<'static>> = Vec::new();

    if !left_items.is_empty() {
        spans.push(Span::styled("\u{2039} ", dot_style));
        for (i, item) in left_items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ", neighbor_style));
            }
            spans.push(Span::styled(item.clone(), neighbor_style));
        }
        spans.push(Span::raw(" "));
    } else {
        spans.push(Span::raw("  "));
    }

    spans.push(Span::styled(
        active_text,
        if selected {
            active_style
        } else {
            Style::default().fg(Color::Cyan)
        },
    ));

    if !right_items.is_empty() {
        spans.push(Span::raw(" "));
        for (i, item) in right_items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ", neighbor_style));
            }
            spans.push(Span::styled(item.clone(), neighbor_style));
        }
        spans.push(Span::styled(" \u{203a}", dot_style));
    }

    Line::from(spans)
}

fn render_slider(
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    precision: usize,
    avail_width: usize,
) -> Line<'static> {
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let val_text = if precision == 0 {
        format!("{}{unit}", value as i64)
    } else {
        format!("{:.prec$}{unit}", value, prec = precision)
    };
    let bar_width = avail_width.saturating_sub(val_text.chars().count() + 2);
    let filled = (t * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    Line::from(vec![
        Span::styled(val_text, Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled("\u{2501}".repeat(filled), Style::default().fg(Color::Cyan)),
        Span::styled(
            "\u{2500}".repeat(empty),
            Style::default().fg(Color::Indexed(238)),
        ),
    ])
}

fn render_toggle(enabled: bool) -> Line<'static> {
    if enabled {
        Line::from(Span::styled("On", Style::default().fg(Color::Cyan).bold()))
    } else {
        Line::from(Span::styled("Off", Style::default().fg(Color::DarkGray)))
    }
}

fn format_control_line(
    ctrl: &Control,
    state: &AppState,
    avail: usize,
    selected: bool,
) -> (String, Line<'static>) {
    match ctrl {
        Control::Ratio => (
            "Ratio".to_string(),
            render_ratio_selector(state.ratio_p, state.ratio_q, avail, selected),
        ),
        Control::Orbits => (
            "Orbits".to_string(),
            render_slider(state.num_orbits as f64, 2.0, 32.0, "", 0, avail),
        ),
        Control::Velocity => (
            "Velocity".to_string(),
            render_cycle_selector(
                &state.velocity_mode.to_string(),
                VELOCITY_OPTS,
                avail,
                selected,
            ),
        ),
        Control::Tuning => (
            "Tuning".to_string(),
            render_cycle_selector(&state.tuning.to_string(), TUNING_OPTS, avail, selected),
        ),
        Control::Scale => ("Scale".to_string(), render_toggle(state.scale_enabled)),
        Control::ScaleRoot => (
            "  Root".to_string(),
            render_cycle_selector(&state.scale_root_name, NOTE_OPTS, avail, selected),
        ),
        Control::ScaleType => (
            "  Type".to_string(),
            render_cycle_selector(&state.scale_type.to_string(), SCALE_OPTS, avail, selected),
        ),
        Control::Waveform => (
            "Waveform".to_string(),
            render_cycle_selector(&state.waveform.to_string(), WAVEFORM_OPTS, avail, selected),
        ),
        Control::Subdivisions => (
            "Subdivs".to_string(),
            render_slider(state.subdivisions as f64, 1.0, 12.0, "", 0, avail),
        ),
        Control::Speed => (
            "Speed".to_string(),
            render_slider(state.speed, -5.0, 5.0, "x", 1, avail),
        ),
        Control::BaseFreq => (
            "Base Hz".to_string(),
            render_slider(state.base_freq, 55.0, 880.0, " Hz", 0, avail),
        ),
        Control::Filter => ("Filter".to_string(), render_toggle(state.filter_enabled)),
        Control::Cutoff => (
            "  Cutoff".to_string(),
            render_slider(state.filter_cutoff, 200.0, 12000.0, " Hz", 0, avail),
        ),
        Control::Resonance => (
            "  Q".to_string(),
            render_slider(state.filter_resonance, 0.1, 20.0, "", 1, avail),
        ),
        Control::DelayWet => {
            let pct = (state.delay_wet * 100.0).round();
            (
                "Delay".to_string(),
                render_slider(pct, 0.0, 100.0, "%", 0, avail),
            )
        }
        Control::DelayTime => (
            "  Time".to_string(),
            render_slider(state.delay_time, 0.05, 1.5, "s", 2, avail),
        ),
        Control::DelayFeedback => {
            let pct = (state.delay_feedback * 100.0).round();
            (
                "  Feedbk".to_string(),
                render_slider(pct, 0.0, 95.0, "%", 0, avail),
            )
        }
        Control::Stereo => ("Stereo".to_string(), render_toggle(state.stereo_enabled)),
        Control::Detune => (
            "Detune".to_string(),
            render_slider(state.detune, 0.0, 50.0, " ct", 0, avail),
        ),
        Control::Chord => ("Chord".to_string(), render_toggle(state.chord_enabled)),
        Control::ConvergenceLines => (
            "Conv.Lines".to_string(),
            render_toggle(state.convergence_lines),
        ),
    }
}

pub fn draw_controls(frame: &mut Frame, area: Rect, state: &AppState, cursor: usize) {
    let controls = visible_controls(state);
    let inner_width = (PANEL_WIDTH as usize).saturating_sub(4);
    let label_width = 11;
    let value_avail = inner_width.saturating_sub(label_width + 1);

    let block = Block::default()
        .title(" Rhuidean Studio ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let status = if state.running {
        "\u{25b6} Playing"
    } else {
        "\u{25a0} Stopped"
    };
    let status_style = if state.running {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header_height = 2u16;
    let footer_height = 2u16;
    let scroll_height = inner.height.saturating_sub(header_height + footer_height);

    let header_area = Rect::new(inner.x, inner.y, inner.width, header_height);
    let scroll_area = Rect::new(inner.x, inner.y + header_height, inner.width, scroll_height);
    let footer_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(footer_height),
        inner.width,
        footer_height,
    );

    let header = ratatui::widgets::Paragraph::new(Line::from(Span::styled(
        format!("  {status}"),
        status_style,
    )));
    frame.render_widget(header, header_area);

    let mut items: Vec<ListItem> = Vec::new();
    for (i, ctrl) in controls.iter().enumerate() {
        let is_selected = i == cursor;
        let (label, value_line) = format_control_line(ctrl, state, value_avail, is_selected);
        let prefix = if is_selected { "\u{25b8} " } else { "  " };
        let label_style = if is_selected {
            Style::default().fg(Color::White).bold()
        } else {
            Style::default().fg(Color::Gray)
        };

        let padded_label = format!("{prefix}{:<width$}", label, width = label_width);
        let mut spans = vec![Span::styled(padded_label, label_style)];
        spans.extend(value_line.spans);
        items.push(ListItem::new(Line::from(spans)));
    }

    let scroll_offset = if scroll_height == 0 {
        0
    } else {
        let visible = scroll_height as usize;
        if cursor >= visible {
            cursor - visible + 1
        } else {
            0
        }
    };

    let list = List::new(items);
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(cursor));
    *list_state.offset_mut() = scroll_offset;
    frame.render_stateful_widget(list, scroll_area, &mut list_state);

    let footer = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("?", Style::default().fg(Color::Cyan)),
        Span::styled(" help  ", Style::default().fg(Color::Gray)),
        Span::styled("y", Style::default().fg(Color::Cyan)),
        Span::styled(" yank  ", Style::default().fg(Color::Gray)),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" hide", Style::default().fg(Color::Gray)),
    ]));
    frame.render_widget(footer, footer_area);
}
