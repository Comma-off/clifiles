use crate::app::App;
use crate::fs::entry::EntryKind;
use crate::preview::{self, Preview};
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use ratatui_image::StatefulImage;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("preview");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &mut app.preview {
        Preview::Text(lines) => {
            let text: Vec<Line> = lines
                .iter()
                .skip(app.preview_scroll as usize)
                .map(|l| Line::from(l.clone()))
                .collect();
            frame.render_widget(Paragraph::new(text), inner);
        }
        Preview::Image(proto) => {
            let widget = StatefulImage::default();
            frame.render_stateful_widget(widget, inner, proto.as_mut());
        }
        Preview::Archive(entries) => {
            let text: Vec<Line> = entries
                .iter()
                .skip(app.preview_scroll as usize)
                .map(|l| Line::from(l.clone()))
                .collect();
            frame.render_widget(Paragraph::new(text), inner);
        }
        Preview::Dir(entries) => {
            let text: Vec<Line> = entries
                .iter()
                .skip(app.preview_scroll as usize)
                .map(|e| {
                    let suffix = if matches!(e.kind, EntryKind::Dir) { "/" } else { "" };
                    Line::from(format!("{}{suffix}", e.name))
                })
                .collect();
            frame.render_widget(Paragraph::new(text), inner);
        }
        Preview::Binary { size } => {
            let text = preview::binary_label(*size);
            frame.render_widget(
                Paragraph::new(text).style(ratatui::style::Style::default().fg(Color::DarkGray)),
                inner,
            );
        }
        Preview::Error(msg) => {
            frame.render_widget(
                Paragraph::new(msg.clone()).style(ratatui::style::Style::default().fg(Color::Red)),
                inner,
            );
        }
    }
}
