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
use crate::state::{LogEntry, LogType, Shared, Stats};

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
            stats_guard.log(LogType::Info, "shutdown requested".into());
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
                stats_guard.log(LogType::Info, "ctrl-c".into());
            }
        }
    }
}

fn draw(f: &mut ratatui::Frame, stats: &Stats) {
    let chunks = build_layout(f.area());
    let elapsed = stats.started_at.map(|t| t.elapsed()).unwrap_or_default();
    let ratio = compute_progress_ratio(stats.sets_inserted, stats.total_known);
    let eta_text = compute_eta_text(stats.sets_inserted, stats.total_known, elapsed);
    let throughput = compute_throughput(stats.sets_inserted, elapsed);

    render_title_bar(f, chunks[0], stats);
    render_progress_gauge(f, chunks[1], stats, ratio);
    render_stats_panel(f, chunks[2], stats, elapsed, &eta_text, throughput);
    render_log_panel(f, chunks[3], stats);
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

fn compute_eta_text(sets_inserted: u64, total_known: Option<u64>, elapsed: Duration) -> String {
    match total_known {
        Some(total) if total > sets_inserted && elapsed.as_secs() > 5 => {
            let remaining = total - sets_inserted;
            let sets_per_sec = sets_inserted as f64 / elapsed.as_secs_f64();
            if sets_per_sec > 0.0001 {
                let secs = remaining as f64 / sets_per_sec;
                fmt_dur(Duration::from_secs_f64(secs))
            } else {
                "--".into()
            }
        }
        _ => "--".into(),
    }
}

fn compute_throughput(sets_inserted: u64, elapsed: Duration) -> f64 {
    if elapsed.as_secs() > 0 {
        sets_inserted as f64 / elapsed.as_secs_f64() * 60.0
    } else {
        0.0
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

fn render_stats_panel(f: &mut ratatui::Frame, area: Rect, stats: &Stats, elapsed: Duration, eta_text: &str, throughput: f64) {
    let disk_text = format_disk_usage(stats);
    let lines = vec![
        Line::from(format!("elapsed:  {}", fmt_dur(elapsed))),
        Line::from(format!("eta:      {eta_text}")),
        Line::from(format!("speed:    {:.1} sets/min (limit: {} req/min)", throughput, stats.rate_per_min)),
        Line::from(format!("pages:    {}", stats.pages_fetched)),
        Line::from(format!("sets:     {}", stats.sets_inserted)),
        Line::from(format!("maps:     {}  (skipped {})", stats.maps_inserted, stats.skipped)),
        Line::from(format!("ratings:  {}", stats.ratings_inserted)),
        Line::from(format!("disk:     {}", disk_text)),
        Line::from(format!("errors:   {}", stats.errors)),
    ];
    let widget = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("stats"));
    f.render_widget(widget, area);
}

fn format_disk_usage(stats: &Stats) -> String {
    let rox_text = format_size_with_projection(stats.rox_bytes, stats.rox_saved, stats.total_known, "rox");
    let db_text = format_size_with_projection(stats.db_bytes, stats.maps_inserted, stats.total_known, "db");
    format!("{} | {}", rox_text, db_text)
}

fn format_size_with_projection(bytes: u64, count: u64, total: Option<u64>, label: &str) -> String {
    if bytes == 0 || count == 0 {
        return format!("{}: --", label);
    }
    let current_mb = bytes as f64 / (1024.0 * 1024.0);
    let avg_per_item = bytes as f64 / count as f64;

    match total {
        Some(t) if t > 0 => {
            let projected_bytes = avg_per_item * t as f64;
            let projected_gb = projected_bytes / (1024.0 * 1024.0 * 1024.0);
            format!("{}: {:.0}MB→{:.1}GB", label, current_mb, projected_gb)
        }
        _ => format!("{}: {:.0}MB", label, current_mb),
    }
}

fn render_log_panel(f: &mut ratatui::Frame, area: Rect, stats: &Stats) {
    let start_time = stats.started_at.unwrap_or_else(std::time::Instant::now);
    let lines: Vec<Line> = stats
        .logs
        .iter()
        .rev()
        .take((area.height.saturating_sub(2)) as usize)
        .map(|entry| format_log_entry(entry, start_time))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let title = format!(
        "logs ({}) — last: {}",
        stats.logs.len(),
        stats.last_title.as_deref().unwrap_or("--")
    );
    let widget = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

fn format_log_entry(entry: &LogEntry, start_time: std::time::Instant) -> Line<'static> {
    let elapsed = entry.timestamp.duration_since(start_time);
    let time_str = format!(
        "{:02}:{:02}:{:02}",
        elapsed.as_secs() / 3600,
        (elapsed.as_secs() % 3600) / 60,
        elapsed.as_secs() % 60
    );
    let type_str = match entry.log_type {
        LogType::Network => "NET",
        LogType::Db => "DB ",
        LogType::Calc => "CAL",
        LogType::Info => "INF",
        LogType::Retry => "RTY",
    };
    let type_color = match entry.log_type {
        LogType::Network => Color::Blue,
        LogType::Db => Color::Magenta,
        LogType::Calc => Color::Yellow,
        LogType::Info => Color::Green,
        LogType::Retry => Color::Red,
    };
    Line::from(vec![
        Span::styled(format!("[{time_str}]"), Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(format!("[{type_str}]"), Style::default().fg(type_color)),
        Span::raw(" "),
        Span::raw(entry.message.clone()),
    ])
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
        assert_eq!(compute_eta_text(0, None, Duration::from_secs(60)), "--");
    }

    #[test]
    fn compute_eta_text_returns_dash_when_done() {
        assert_eq!(compute_eta_text(100, Some(100), Duration::from_secs(60)), "--");
    }

    #[test]
    fn compute_eta_text_returns_dash_when_elapsed_too_short() {
        assert_eq!(compute_eta_text(0, Some(100), Duration::from_secs(3)), "--");
    }

    #[test]
    fn compute_eta_text_returns_duration_string() {
        // 30 done in 30s = 1/s, 30 remaining = 30s
        let eta = compute_eta_text(30, Some(60), Duration::from_secs(30));
        assert_eq!(eta, "00m30s");
    }

    #[test]
    fn compute_throughput_returns_sets_per_minute() {
        // 60 sets in 60 seconds = 60 sets/min
        let tp = compute_throughput(60, Duration::from_secs(60));
        assert!((tp - 60.0).abs() < 0.1);
    }
}
