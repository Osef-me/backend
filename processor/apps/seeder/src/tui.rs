use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use ratatui::Terminal;

use crate::config::{RATE_MAX, RATE_MIN};
use crate::limiter::Limiter;
use crate::state::Shared;

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
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                let is_ctrl_c =
                    k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL);
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        let mut s = state.write().await;
                        s.shutdown = true;
                        s.message = Some("shutdown requested".into());
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
                        let mut s = state.write().await;
                        s.paused = !s.paused;
                    }
                    _ => {
                        if is_ctrl_c {
                            let mut s = state.write().await;
                            s.shutdown = true;
                            s.message = Some("ctrl-c".into());
                        }
                    }
                }
            }
        }

        {
            let s = state.read().await;
            if s.shutdown && s.done {
                break;
            }
        }
    }
    Ok(())
}

fn draw(f: &mut ratatui::Frame, s: &crate::state::Stats) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(11),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    let elapsed = s.started_at.map(|t| t.elapsed()).unwrap_or_default();
    let total = s.total_known;
    let done = s.sets_inserted;
    let ratio = match total {
        Some(t) if t > 0 => (done as f64 / t as f64).clamp(0.0, 1.0),
        _ => 0.0,
    };
    let eta_txt = match total {
        Some(t) if t > done && s.rate_per_min > 0 => {
            let remaining = t - done;
            let per_sec = s.rate_per_min as f64 / 60.0;
            let secs = remaining as f64 / per_sec.max(0.0001);
            fmt_dur(Duration::from_secs_f64(secs))
        }
        _ => "--".into(),
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled("osu! mania seeder", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  — "),
        Span::styled(
            if s.paused { "PAUSED" } else if s.shutdown { "STOPPING" } else { "RUNNING" },
            Style::default().fg(if s.paused { Color::Yellow } else if s.shutdown { Color::Red } else { Color::Green }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("progress (sets)"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(ratio)
        .label(match total {
            Some(t) => format!("{}/{}  ({:.1}%)", done, t, ratio * 100.0),
            None => format!("{} / calculating…", done),
        });
    f.render_widget(gauge, chunks[1]);

    let stats_lines = vec![
        Line::from(format!("elapsed:  {}", fmt_dur(elapsed))),
        Line::from(format!("eta:      {}", eta_txt)),
        Line::from(format!("rate:     {} req/min (min {} / max {})", s.rate_per_min, RATE_MIN, RATE_MAX)),
        Line::from(format!("pages:    {}", s.pages_fetched)),
        Line::from(format!("sets:     {}", s.sets_inserted)),
        Line::from(format!("maps:     {}  (skipped {})", s.maps_inserted, s.skipped)),
        Line::from(format!("ratings:  {}", s.ratings_inserted)),
        Line::from(format!(".rox:     {}", s.rox_saved)),
        Line::from(format!("errors:   {}", s.errors)),
    ];
    let stats = Paragraph::new(stats_lines)
        .block(Block::default().borders(Borders::ALL).title("stats"));
    f.render_widget(stats, chunks[2]);

    let last = Paragraph::new(vec![
        Line::from(format!("last: {}", s.last_title.clone().unwrap_or_else(|| "--".into()))),
        Line::from(format!("cursor: {}", s.last_cursor.clone().unwrap_or_else(|| "--".into()))),
        Line::from(format!("msg: {}", s.message.clone().unwrap_or_default())),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title("last"));
    f.render_widget(last, chunks[3]);

    let help = Paragraph::new(Line::from(
        "q/Esc quit+dump · +/- rate ±10 · p pause · Ctrl+C stop",
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[4]);
}

fn fmt_dur(d: Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}h{m:02}m{s:02}s")
    } else {
        format!("{m:02}m{s:02}s")
    }
}
