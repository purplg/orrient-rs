use std::mem;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

pub struct Textbox<'a> {
    style: Style,
    block: Option<Block<'a>>,
}

impl<'a> Textbox<'a> {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> StatefulWidget for Textbox<'a> {
    type State = TextboxState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        let text_box_area = match self.block.take() {
            Some(block) => {
                let inner_area = block.inner(area);
                block.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_box_area.width < 1 || text_box_area.height < 1 {
            return;
        }

        buf.set_stringn(
            text_box_area.x,
            text_box_area.y,
            state.content(),
            text_box_area.width as usize,
            self.style,
        );
    }
}

impl<'a> Widget for Textbox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        <Self as StatefulWidget>::render(self, area, buf, &mut TextboxState::default());
    }
}

#[derive(Default)]
pub struct TextboxState {
    content: String,
    cursor_position: u16,
}

impl TextboxState {
    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn cursor_position(&self) -> u16 {
        self.content.len() as u16
    }

    pub fn insert_character(&mut self, c: char) {
        self.content.insert(self.cursor_position() as usize, c);
        self.cursor_position += 1;
    }

    pub fn remove_character(&mut self) {
        if self.cursor_position() > 0 {
            self.content.remove(self.cursor_position() as usize - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn take(&mut self) -> String {
        mem::take(&mut self.content)
    }
}
