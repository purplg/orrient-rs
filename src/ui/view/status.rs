use crate::events::{InputEvent, ViewEvent};
use tui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::View;

pub struct StatusView {
    message: String,
}

impl StatusView {
    pub fn new() -> Self {
        StatusView {
            message: String::default(),
        }
    }
}

impl View for StatusView {
    fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        frame.render_widget(
            Paragraph::new(self.message.as_str()).block(Block::default().borders(Borders::TOP)),
            area,
        );
    }

    fn handle_input_event(&mut self, _: &InputEvent) -> bool {
        false
    }

    fn handle_view_event(&mut self, view_event: &ViewEvent) {
        if let ViewEvent::UpdateStatus(message) = view_event {
            self.message = message.clone();
        }
    }
}
