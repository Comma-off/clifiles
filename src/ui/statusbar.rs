use crate::app::{App, Mode, PendingOp};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, status_area: Rect, prompt_area: Rect) {
    let status_text = match app.selected_entry() {
        Some(entry) => format!(
            "{} entries | selected: {}",
            app.entries.len(),
            entry.name
        ),
        None => format!("{} entries", app.entries.len()),
    };
    frame.render_widget(
        Paragraph::new(Line::from(status_text)).style(Style::default().fg(Color::DarkGray)),
        status_area,
    );

    let prompt_text = match &app.mode {
        Mode::Normal => app
            .message
            .clone()
            .unwrap_or_else(|| "j/k move  l/h in/out  d:del r:ren m:mkdir t:touch y:copy x:cut p:paste s:sort .:hidden q:quit".to_string()),
        Mode::Prompt { op, input } => {
            let label = match op {
                PendingOp::Rename => "rename",
                PendingOp::Mkdir => "new dir",
                PendingOp::Touch => "new file",
                _ => "input",
            };
            format!("{label}: {input}")
        }
        Mode::Confirm { op } => match op {
            PendingOp::Delete => "delete selected? (y/n)".to_string(),
            PendingOp::Paste { cut: true } => "move here? (y/n)".to_string(),
            PendingOp::Paste { cut: false } => "copy here? (y/n)".to_string(),
            _ => "confirm? (y/n)".to_string(),
        },
    };
    frame.render_widget(Paragraph::new(prompt_text), prompt_area);
}
