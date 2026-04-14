mod app;
mod audio;
mod renderer;
mod ui;

use std::io;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::Terminal;

use rhuidean_studio_core::scale;

use app::AppState;
use audio::TriggerMsg;
use renderer::FlashState;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match result {
        Ok(Some(yanked)) => {
            println!("{yanked}");
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<Option<String>> {
    let mut state = AppState::default();
    let mut flash = FlashState::new(state.num_orbits);

    let (tx, rx) = mpsc::channel::<TriggerMsg>();
    let audio_params = Arc::new(Mutex::new(state.audio_params()));
    let _stream = audio::spawn_audio_thread(rx, audio_params.clone());

    let tick_rate = Duration::from_millis(16);
    let mut last_tick = Instant::now();
    let mut reset_primed = false;
    let mut reset_time = Instant::now();
    let mut share_msg: Option<(String, Instant)> = None;
    let mut last_yank: Option<String> = None;
    let mut show_help = false;

    loop {
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }

                if show_help {
                    show_help = false;
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(last_yank),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(last_yank);
                    }
                    KeyCode::Char('?') => {
                        show_help = true;
                    }
                    KeyCode::Char(' ') => {
                        state.running = !state.running;
                    }
                    KeyCode::Tab => {
                        state.show_controls = !state.show_controls;
                    }
                    KeyCode::Char('r') => {
                        if reset_primed && reset_time.elapsed() < Duration::from_millis(500) {
                            state.defaults();
                            state.rebuild();
                            flash = FlashState::new(state.num_orbits);
                            sync_audio_params(&state, &audio_params);
                            reset_primed = false;
                        } else {
                            state.system.reset();
                            flash.reset();
                            reset_primed = true;
                            reset_time = Instant::now();
                        }
                    }
                    KeyCode::Char('y') if state.show_controls => {
                        let cmd = state.share_command();
                        yank_to_clipboard(&cmd);
                        last_yank = Some(cmd.clone());
                        share_msg = Some((cmd, Instant::now()));
                    }
                    KeyCode::Char('j') | KeyCode::Down if state.show_controls => {
                        let max = ui::control_count(&state);
                        if max > 0 {
                            state.cursor = (state.cursor + 1) % max;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up if state.show_controls => {
                        let max = ui::control_count(&state);
                        if max > 0 {
                            state.cursor = (state.cursor + max - 1) % max;
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right if state.show_controls => {
                        state.adjust(1);
                        flash.set_num_orbits(state.num_orbits);
                        sync_audio_params(&state, &audio_params);
                    }
                    KeyCode::Char('h') | KeyCode::Left if state.show_controls => {
                        state.adjust(-1);
                        flash.set_num_orbits(state.num_orbits);
                        sync_audio_params(&state, &audio_params);
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let dt = last_tick.elapsed().as_secs_f64();
            last_tick = Instant::now();

            if reset_primed && reset_time.elapsed() >= Duration::from_millis(500) {
                reset_primed = false;
            }

            if state.running {
                let scaled_dt = dt * state.speed;
                let events = state.system.tick(scaled_dt);
                let num_orbits = state.system.orbits.len();
                let is_convergence = events.len() == num_orbits && num_orbits > 1;

                for event in &events {
                    let freq = if state.scale_enabled {
                        scale::degree_frequency(
                            event.orbit_index,
                            state.scale_root_hz,
                            &state.scale_type,
                        )
                    } else {
                        state
                            .tuning
                            .frequency(event.orbit_index, num_orbits, state.base_freq)
                    };
                    let _ = tx.send(TriggerMsg {
                        frequency: freq,
                        orbit_index: event.orbit_index,
                        num_orbits,
                        is_convergence,
                    });
                    flash.trigger_flash(event.orbit_index);
                }

                if events.len() >= 2 {
                    let indices: Vec<usize> = events.iter().map(|e| e.orbit_index).collect();
                    flash.trigger_convergence(indices);
                }
            }

            flash.tick(dt);

            terminal.draw(|frame| {
                let area = frame.area();

                if state.show_controls {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(10), Constraint::Length(ui::PANEL_WIDTH)])
                        .split(area);

                    renderer::draw_visualization(
                        frame,
                        chunks[0],
                        &state.system,
                        &flash,
                        state.convergence_lines,
                    );
                    ui::draw_controls(frame, chunks[1], &state, state.cursor);
                } else {
                    renderer::draw_visualization(
                        frame,
                        area,
                        &state.system,
                        &flash,
                        state.convergence_lines,
                    );
                }

                if let Some((ref msg, time)) = share_msg {
                    if time.elapsed() < Duration::from_secs(3) {
                        let popup_area = Rect::new(
                            area.x + 1,
                            area.height.saturating_sub(2),
                            area.width.saturating_sub(2).min(msg.len() as u16 + 2),
                            1,
                        );
                        frame.render_widget(
                            Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Cyan)),
                            popup_area,
                        );
                    }
                }

                if show_help {
                    draw_help(frame, area);
                }
            })?;
        }
    }
}

fn sync_audio_params(state: &AppState, params: &Arc<Mutex<audio::AudioParams>>) {
    if let Ok(mut p) = params.lock() {
        *p = state.audio_params();
    }
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help_w = 48u16.min(area.width.saturating_sub(4));
    let help_h = 22u16.min(area.height.saturating_sub(4));
    let help_area = Rect::new(
        area.x + (area.width.saturating_sub(help_w)) / 2,
        area.y + (area.height.saturating_sub(help_h)) / 2,
        help_w,
        help_h,
    );

    frame.render_widget(Clear, help_area);

    let text = "\
Space       Play / Pause
Tab         Toggle controls panel
r           Reset orbits
r r         Reset to defaults (double-tap)
j / Down    Move cursor down
k / Up      Move cursor up
h / Left    Decrease / previous
l / Right   Increase / next
y           Yank share command
?           Toggle this help
q / Esc     Quit

Controls are navigated with j/k to
select a parameter, then h/l to
adjust its value. Toggle options
flip on any direction press.";

    let block = Block::default()
        .title(" Keys ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::uniform(1));

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(paragraph, help_area);
}

fn yank_to_clipboard(text: &str) {
    use std::process::{Command, Stdio};

    let candidates: &[&str] = if cfg!(target_os = "macos") {
        &["pbcopy"]
    } else {
        &["xclip -selection clipboard", "xsel --clipboard --input"]
    };

    for cmd in candidates {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if let Ok(mut child) = Command::new(parts[0])
            .args(&parts[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return;
        }
    }
}
