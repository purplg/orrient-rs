// A view is a collection of components or widgets to display to the terminal buffer
pub mod achievements;
pub mod dailies;
pub mod status;
pub mod timer;
pub mod tracks;

use tui::{backend::Backend, layout::Rect, Frame};

use crate::{events::ViewEvent, input::InputEvent};

pub trait View {
    fn draw<B: Backend>(&mut self, _: &mut Frame<B>, _: Rect);

    fn handle_input_event(&mut self, _: &InputEvent) -> bool;
    fn handle_view_event(&mut self, _: &ViewEvent);
}
