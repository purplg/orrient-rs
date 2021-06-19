use std::rc::Rc;

use crate::{
    events::{InputEvent, ViewEvent},
    state::AppState,
};
use tui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::View;

pub struct StatusView {
    app_state: Rc<AppState>,
}

impl StatusView {
    pub fn new(app_state: Rc<AppState>) -> Self {
        StatusView { app_state }
    }
}

impl View for StatusView {
    fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        frame.render_widget(
            Paragraph::new(self.app_state.status()).block(Block::default().borders(Borders::TOP)),
            area,
        );
    }

    fn handle_input_event(&mut self, _: &InputEvent) -> bool {
        false
    }

    fn handle_view_event(&mut self, _: &ViewEvent) {}
}
