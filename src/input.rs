use crate::app::{App, Mode, PendingOp};
use anyhow::Result;
use crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::time::{Duration, Instant};

const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);

pub enum Signal {
    Continue,
    Redraw,
    PreviewDirty,
    Quit,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<Signal> {
    match &app.mode {
        Mode::Normal => handle_normal_key(app, key),
        Mode::Prompt { .. } => handle_prompt_key(app, key),
        Mode::Confirm { .. } => handle_confirm_key(app, key),
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) -> Result<Signal> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(Signal::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Signal::Quit)
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_selection(1);
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_selection(-1);
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('g') => {
            app.selected = 0;
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('G') => {
            app.selected = app.entries.len().saturating_sub(1);
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => {
            app.enter_selected()?;
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
            app.go_up()?;
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('.') => {
            app.toggle_hidden()?;
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('d') => app.request_delete(),
        KeyCode::Char('r') => app.request_rename(),
        KeyCode::Char('m') => app.start_prompt(PendingOp::Mkdir, ""),
        KeyCode::Char('t') => app.start_prompt(PendingOp::Touch, ""),
        KeyCode::Char('y') => app.copy_to_clipboard(false),
        KeyCode::Char('x') => app.copy_to_clipboard(true),
        KeyCode::Char('p') => app.request_paste(),
        KeyCode::Char('s') => app.cycle_sort()?,
        KeyCode::Char('J') => {
            app.preview_scroll = app.preview_scroll.saturating_add(1);
        }
        KeyCode::Char('K') => {
            app.preview_scroll = app.preview_scroll.saturating_sub(1);
        }
        _ => return Ok(Signal::Continue),
    }
    Ok(Signal::Redraw)
}

fn handle_prompt_key(app: &mut App, key: KeyEvent) -> Result<Signal> {
    let Mode::Prompt { input, .. } = &mut app.mode else {
        return Ok(Signal::Continue);
    };
    match key.code {
        KeyCode::Esc => app.cancel_mode(),
        KeyCode::Enter => {
            app.run_prompt_submit()?;
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Backspace => {
            input.pop();
        }
        KeyCode::Char(c) => input.push(c),
        _ => return Ok(Signal::Continue),
    }
    Ok(Signal::Redraw)
}

fn handle_confirm_key(app: &mut App, key: KeyEvent) -> Result<Signal> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.run_confirm_submit()?;
            return Ok(Signal::PreviewDirty);
        }
        KeyCode::Char('n') | KeyCode::Esc => app.cancel_mode(),
        _ => return Ok(Signal::Continue),
    }
    Ok(Signal::Redraw)
}

/// `list_top_row` / `list_area_row_count` describe the visible window of the
/// file list pane so a click can be mapped back to an entry index.
pub fn handle_mouse(
    app: &mut App,
    ev: MouseEvent,
    list_top_row: u16,
    list_area_row_count: u16,
) -> Signal {
    match ev.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if ev.row >= list_top_row && ev.row < list_top_row + list_area_row_count {
                let idx = (ev.row - list_top_row) as usize;
                if idx < app.entries.len() {
                    let now = Instant::now();
                    let is_double_click = app
                        .last_click
                        .map(|(t, last_idx)| last_idx == idx && now.duration_since(t) < DOUBLE_CLICK_WINDOW)
                        .unwrap_or(false);
                    app.selected = idx;
                    app.last_click = Some((now, idx));
                    if is_double_click {
                        app.last_click = None;
                        let _ = app.enter_selected();
                    }
                    return Signal::PreviewDirty;
                }
            }
        }
        MouseEventKind::ScrollDown => {
            app.move_selection(1);
            return Signal::PreviewDirty;
        }
        MouseEventKind::ScrollUp => {
            app.move_selection(-1);
            return Signal::PreviewDirty;
        }
        _ => {}
    }
    Signal::Continue
}
