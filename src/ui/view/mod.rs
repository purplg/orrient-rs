// A view is a collection of components or widgets to display to the terminal buffer
pub mod achievements;
pub mod bookmarks;
pub mod dailies;
pub mod status;
pub mod timer;
pub mod tracks;

use std::io::Stdout;

use tui::{backend::CrosstermBackend, layout::Rect, Frame};

use crate::{events::Event, input::InputEvent};

pub trait View {
    fn name(&self) -> &'static str;

    fn draw(&mut self, _: &mut Frame<CrosstermBackend<Stdout>>, _: Rect);

    fn handle_input_event(&mut self, _: &InputEvent) -> bool;
    fn handle_event(&mut self, _: &Event);
}
