use std::{cmp::min, mem};

use crossterm::event::KeyCode;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::input::{InputEvent, InputKind};

use super::list_selection::CursorMovement;

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
        let area = match self.block.take() {
            Some(block) => {
                let inner_area = block.inner(area);
                block.render(area, buf);
                inner_area
            }
            None => area,
        };

        if area.width < 1 || area.height < 1 {
            return;
        }

        buf.set_stringn(
            area.x,
            area.y,
            state.content(),
            area.width as usize,
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
    /// Clears the content of the textbox
    pub fn clear(&mut self) {
        self.content.clear();
    }

    /// Get a reference to the content
    pub fn content(&self) -> &String {
        &self.content
    }

    /// Get the current position of the cursor
    pub fn cursor_position(&self) -> u16 {
        self.cursor_position
    }

    /// Insert a character at the current position
    pub fn insert_character(&mut self, c: char) {
        self.content.insert(self.cursor_position() as usize, c);
        self.cursor_position += 1;
    }

    /// Remove the character at the current position
    pub fn remove_character(&mut self) {
        if self.cursor_position() > 0 {
            self.content.remove(self.cursor_position() as usize - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor(&mut self, movement: CursorMovement) {
        match movement {
            CursorMovement::Left(amount) => {
                self.cursor_position -= min(self.cursor_position, amount as u16);
            }
            CursorMovement::Right(amount) => {
                self.cursor_position += min(
                    amount as u16,
                    self.content.len() as u16 - self.cursor_position,
                );
            }
            _ => {}
        }
    }

    pub fn handle_input(&mut self, event: &InputEvent) -> bool {
        if let Some(key_code) = event.key_code {
            match key_code {
                KeyCode::Char(letter) => {
                    self.insert_character(letter);
                    return true;
                }
                KeyCode::Backspace => {
                    self.remove_character();
                    return true;
                }
                _ => {}
            }
        }

        match event.input {
            InputKind::MoveLeft(amount) => {
                self.move_cursor(CursorMovement::Left(amount));
                return true;
            }
            InputKind::MoveRight(amount) => {
                self.move_cursor(CursorMovement::Right(amount));
                return true;
            }
            _ => {}
        }
        false
    }

    /// 'Take' and return the content of the textbox leaving it empty
    ///
    /// Useful when you're completing a form and getting the contents without requiring unnecessary
    /// allocations    pub fn take(&mut self) -> String {
    pub fn take(&mut self) -> String {
        mem::take(&mut self.content)
    }
}
