use std::io::{self, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind,
};
use crossterm::execute;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{
    self, BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen,
    LeaveAlternateScreen,
};

use crate::core::process;
use crate::core::style::Style;
use crate::core::theme;

use super::diffview;

struct Guard;

impl Guard {
    fn enter() -> Result<Self, String> {
        terminal::enable_raw_mode().map_err(|e| format!("failed to enable raw mode: {e}"))?;
        execute!(io::stdout(), EnterAlternateScreen, Hide, EnableMouseCapture)
            .map_err(|e| format!("failed to enter alternate screen: {e}"))?;
        Ok(Self)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            Show,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}

pub(super) fn run(theme_name: Option<&str>, staged: bool) -> Result<(), String> {
    let style = Style::for_stdout();
    let theme = theme::active(theme_name);
    let mut scroll = 0usize;

    let mut args = vec!["diff", "--no-color", "--unified=3"];
    if staged {
        args.push("--staged");
    }
    let diff =
        process::run_capture("git", &args).map_err(|e| format!("failed to run git diff: {e}"))?;

    let _guard = Guard::enter()?;

    loop {
        let (w, h) = terminal::size().map_err(|e| e.to_string())?;
        let w = w as usize;
        let h = h as usize;

        let mut out = io::stdout();
        queue!(
            out,
            BeginSynchronizedUpdate,
            MoveTo(0, 0),
            Clear(ClearType::All)
        )
        .map_err(|e| e.to_string())?;

        let title = if staged {
            "Git Diff TUI (staged)"
        } else {
            "Git Diff TUI"
        };
        draw_line(&mut out, 0, &style.paint(theme.title, pad(title, w)))?;
        draw_line(
            &mut out,
            1,
            &style.paint(
                theme.subtle,
                pad("j/k or mouse wheel scroll  g/G top/bottom  q/Esc quit", w),
            ),
        )?;

        let body_h = h.saturating_sub(3);
        if diff.trim().is_empty() {
            draw_line(
                &mut out,
                2,
                &style.paint(theme.info, pad("No changes to diff.", w)),
            )?;
        } else {
            let rendered = diffview::render_side_by_side(&diff, &style, theme, w);
            let max_scroll = rendered.len().saturating_sub(body_h);
            if scroll > max_scroll {
                scroll = max_scroll;
            }

            for (row, line) in rendered.iter().skip(scroll).take(body_h).enumerate() {
                draw_line(&mut out, (2 + row) as u16, line)?;
            }
        }

        queue!(out, EndSynchronizedUpdate).map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;

        if !event::poll(Duration::from_millis(120)).map_err(|e| e.to_string())? {
            continue;
        }

        match event::read().map_err(|e| e.to_string())? {
            Event::Key(k) => match k.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('j') | KeyCode::Down => scroll = scroll.saturating_add(1),
                KeyCode::Char('k') | KeyCode::Up => scroll = scroll.saturating_sub(1),
                KeyCode::Char('g') => scroll = 0,
                KeyCode::Char('G') => scroll = usize::MAX,
                _ => {}
            },
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollUp => scroll = scroll.saturating_sub(2),
                MouseEventKind::ScrollDown => scroll = scroll.saturating_add(2),
                _ => {}
            },
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(())
}

fn draw_line(out: &mut io::Stdout, y: u16, line: &str) -> Result<(), String> {
    queue!(
        out,
        MoveTo(0, y),
        Clear(ClearType::CurrentLine),
        Print(line)
    )
    .map_err(|e| e.to_string())
}

fn pad(s: &str, width: usize) -> String {
    let truncated = if s.chars().count() > width {
        let mut out = String::new();
        for (i, ch) in s.chars().enumerate() {
            if i + 1 >= width {
                out.push('…');
                break;
            }
            out.push(ch);
        }
        out
    } else {
        s.to_string()
    };

    if truncated.chars().count() >= width {
        truncated
    } else {
        format!("{truncated:<width$}")
    }
}
