use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use ratatui::Terminal;

use crate::config::{RATE_MAX, RATE_MIN};
use crate::limiter::Limiter;
use crate::state::{Shared, Stats};

pub async fn run(state: Shared, limiter: Arc<Limiter>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let res = event_loop(&mut term, state.clone(), limiter.clone()).await;

    disable_raw_mode()?;
    term.backend_mut().execute(LeaveAlternateScreen)?;
    term.show_cursor()?;
    res
}

async fn event_loop<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    state: Shared,
    limiter: Arc<Limiter>,
) -> Result<()> {
    loop {
        {
            let snap = state.read().await.clone();
            term.draw(|f| draw(f, &snap))?;
            if snap.shutdown && snap.done {
                break;
            }
        }

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(key, &state, &limiter).await;
                }
            }
        }

        if state.read().await.shutdown && state.read().await.done {
            break;
        }
    }
    Ok(())
}

async fn handle_key_event(key: KeyEvent, state: &Shared, limiter: &Arc<Limiter>) {
    let is_ctrl_c = key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            let mut stats_guard = state.write().await;
            stats_guard.shutdown = true;
            stats_guard.message = Some("shutdown requested".into());
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            let new_rate = (limiter.rate() + 10).min(RATE_MAX);
            limiter.set_rate(new_rate);
            state.write().await.rate_per_min = new_rate;
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            let new_rate = limiter.rate().saturating_sub(10).max(RATE_MIN);
            limiter.set_rate(new_rate);
            state.write().await.rate_per_min = new_rate;
        }
        KeyCode::Char('p') => {
            let mut stats_guard = state.write().await;
            stats_guard.paused = !stats_guard.paused;
        }
        _ => {
            if is_ctrl_c {
                let mut stats_guard = state.write().await;
                stats_guard.shutdown = true;
                stats_guard.message = Some("ctrl-c".into());
            }
        }
    }
}

fn draw(f: &mut ratatui::Frame, stats: &Stats) {
    let chunks = build_layout(f.area());
    let elapsed = stats.started_at.map(|t| t.elapsed()).unwrap_or_default();
    let ratio = compute_progress_ratio(stats.sets_inserted, stats.total_known);
    let eta_text = compute_eta_text(stats.sets_inserted, stats.total_known, stats.rate_per_min);

    render_title_bar(f, chunks[0], stats);
    render_progress_gauge(f, chunks[1], stats, ratio);
    render_stats_panel(f, chunks[2], stats, elapsed, &eta_text);
    render_last_panel(f, chunks[3], stats);
    render_help_bar(f, chunks[4]);
}

fn build_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(11),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area)
}

fn compute_progress_ratio(sets_inserted: u64, total_known: Option<u64>) -> f64 {
    match total_known {
        Some(total) if total > 0 => (sets_inserted as f64 / total as f64).clamp(0.0, 1.0),
        _ => 0.0,
    }
}

fn compute_eta_text(sets_inserted: u64, total_known: Option<u64>, rate_per_min: u32) -> String {
    match total_known {
        Some(total) if total > sets_inserted && rate_per_min > 0 => {
            let remaining = total - sets_inserted;
            let per_sec = rate_per_min as f64 / 60.0;
            let secs = remaining as f64 / per_sec.max(0.0001);
            fmt_dur(Duration::from_secs_f64(secs))
        }
        _ => "--".into(),
    }
}

fn render_title_bar(f: &mut ratatui::Frame, area: Rect, stats: &Stats) {
    let status_label = if stats.paused { "PAUSED" } else if stats.shutdown { "STOPPING" } else { "RUNNING" };
    let status_color = if stats.paused { Color::Yellow } else if stats.shutdown { Color::Red } else { Color::Green };
    let widget = Paragraph::new(Line::from(vec![
        Span::styled("osu! mania seeder", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  — "),
        Span::styled(status_label, Style::default().fg(status_color)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(widget, area);
}

fn render_progress_gauge(f: &mut ratatui::Frame, area: Rect, stats: &Stats, ratio: f64) {
    let label = match stats.total_known {
        Some(total) => format!("{}/{total}  ({:.1}%)", stats.sets_inserted, ratio * 100.0),
        None => format!("{} / calculating…", stats.sets_inserted),
    };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("progress (sets)"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(ratio)
        .label(label);
    f.render_widget(gauge, area);
}

fn render_stats_panel(f: &mut ratatui::Frame, area: Rect, stats: &Stats, elapsed: Duration, eta_text: &str) {
    let lines = vec![
        Line::from(format!("elapsed:  {}", fmt_dur(elapsed))),
        Line::from(format!("eta:      {eta_text}")),
        Line::from(format!("rate:     {} req/min (min {} / max {})", stats.rate_per_min, RATE_MIN, RATE_MAX)),
        Line::from(format!("pages:    {}", stats.pages_fetched)),
        Line::from(format!("sets:     {}", stats.sets_inserted)),
        Line::from(format!("maps:     {}  (skipped {})", stats.maps_inserted, stats.skipped)),
        Line::from(format!("ratings:  {}", stats.ratings_inserted)),
        Line::from(format!(".rox:     {}", stats.rox_saved)),
        Line::from(format!("errors:   {}", stats.errors)),
    ];
    let widget = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("stats"));
    f.render_widget(widget, area);
}

fn render_last_panel(f: &mut ratatui::Frame, area: Rect, stats: &Stats) {
    let widget = Paragraph::new(vec![
        Line::from(format!("last:   {}", stats.last_title.as_deref().unwrap_or("--"))),
        Line::from(format!("cursor: {}", stats.last_cursor.as_deref().unwrap_or("--"))),
        Line::from(format!("msg:    {}", stats.message.as_deref().unwrap_or(""))),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title("last"));
    f.render_widget(widget, area);
}

fn render_help_bar(f: &mut ratatui::Frame, area: Rect) {
    let widget = Paragraph::new(Line::from(
        "q/Esc quit+dump · +/- rate ±10 · p pause · Ctrl+C stop",
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(widget, area);
}

fn fmt_dur(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{hours}h{minutes:02}m{seconds:02}s")
    } else {
        format!("{minutes:02}m{seconds:02}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_dur_formats_seconds_only() {
        assert_eq!(fmt_dur(Duration::from_secs(45)), "00m45s");
    }

    #[test]
    fn fmt_dur_formats_minutes_and_seconds() {
        assert_eq!(fmt_dur(Duration::from_secs(125)), "02m05s");
    }

    #[test]
    fn fmt_dur_formats_hours() {
        assert_eq!(fmt_dur(Duration::from_secs(3661)), "1h01m01s");
    }

    #[test]
    fn compute_progress_ratio_zero_when_no_total() {
        assert_eq!(compute_progress_ratio(50, None), 0.0);
    }

    #[test]
    fn compute_progress_ratio_zero_when_total_is_zero() {
        assert_eq!(compute_progress_ratio(0, Some(0)), 0.0);
    }

    #[test]
    fn compute_progress_ratio_clamped_to_one() {
        assert_eq!(compute_progress_ratio(200, Some(100)), 1.0);
    }

    #[test]
    fn compute_progress_ratio_half() {
        assert!((compute_progress_ratio(50, Some(100)) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_eta_text_returns_dash_when_no_total() {
        assert_eq!(compute_eta_text(0, None, 60), "--");
    }

    #[test]
    fn compute_eta_text_returns_dash_when_done() {
        assert_eq!(compute_eta_text(100, Some(100), 60), "--");
    }

    #[test]
    fn compute_eta_text_returns_dash_when_rate_zero() {
        assert_eq!(compute_eta_text(0, Some(100), 0), "--");
    }

    #[test]
    fn compute_eta_text_returns_duration_string() {
        // 60 remaining at 60/min = 60 seconds
        let eta = compute_eta_text(0, Some(60), 60);
        assert_eq!(eta, "01m00s");
    }
}
