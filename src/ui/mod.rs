mod listing;
mod preview;
mod statusbar;

use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

pub struct Layouts {
    pub list_inner_top: u16,
    pub list_inner_rows: u16,
}

pub fn draw(frame: &mut Frame, app: &mut App) -> Layouts {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(root[0]);

    let list_area = main[0];
    listing::draw(frame, app, list_area);
    preview::draw(frame, app, main[1]);
    statusbar::draw(frame, app, root[1], root[2]);

    Layouts {
        list_inner_top: list_area.y + 1,
        list_inner_rows: list_area.height.saturating_sub(2),
    }
}
