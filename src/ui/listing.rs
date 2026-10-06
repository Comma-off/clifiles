use crate::app::App;
use crate::fs::entry::{human_size, EntryKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let colors = &app.config.colors;
    let title = app.cwd.to_string_lossy().into_owned();
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| {
            let color = match entry.kind {
                EntryKind::Dir => colors.dir.0,
                EntryKind::Symlink => colors.symlink.0,
                EntryKind::File => colors.file.0,
            };
            let marker = match entry.kind {
                EntryKind::Dir => "/",
                EntryKind::Symlink => "@",
                EntryKind::File => "",
            };
            let size = if entry.is_dir() {
                String::new()
            } else {
                human_size(entry.size)
            };
            let line = Line::from(vec![
                Span::styled(format!("{}{marker}", entry.name), Style::default().fg(color)),
                Span::raw(format!(" {size:>10}")),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let mut highlight = Style::default();
    highlight = match (colors.highlight_fg, colors.highlight_bg) {
        (None, None) => highlight.add_modifier(Modifier::REVERSED),
        (fg, bg) => {
            if let Some(fg) = fg {
                highlight = highlight.fg(fg.0);
            }
            if let Some(bg) = bg {
                highlight = highlight.bg(bg.0);
            }
            highlight
        }
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(highlight);

    frame.render_stateful_widget(list, area, &mut state);
}
