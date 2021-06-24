use std::iter;

use crossterm::event::KeyCode;
use tui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{
    events::{InputEvent, InputKind},
    tracks::Track,
    ui::widget::{
        checkbox::{Checkbox, CheckboxState},
        list_selection::{CursorMovement, ListSelection},
        text_box::{Textbox, TextboxState},
    },
};

pub struct CustomTrackPopupState {
    textbox_state: TextboxState,
    checkbox_state: CheckboxState,
    list_state: ListState,
}

impl Default for CustomTrackPopupState {
    fn default() -> Self {
        let mut popup = Self {
            textbox_state: TextboxState::default(),
            checkbox_state: CheckboxState::default(),
            list_state: ListState::default(),
        };
        popup.list_state.select(Some(0));
        popup
    }
}

impl CustomTrackPopupState {
    pub fn cancel(&mut self) {
        self.textbox_state = TextboxState::default();
        self.checkbox_state = CheckboxState::default();
        self.list_state = ListState::default();
        self.list_state.select(Some(0));
    }

    pub fn finish(&mut self) -> Track {
        let track = Track::Custom(self.textbox_state.take());
        self.cancel();
        track
    }

    pub fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let (width, height) = (50, 10);
        if area.width < width || area.height < height {
            return;
        }

        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;

        let style = Style::default().add_modifier(Modifier::REVERSED);
        let area = Rect::new(x, y, width, height);
        let background = Block::default()
            .borders(Borders::ALL)
            .title("Add custom item")
            .style(style);
        frame.render_widget(Clear, area);
        frame.render_widget(background, area);

        let area = area.inner(&Margin {
            vertical: 2,
            horizontal: 2,
        });

        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(2), Constraint::Length(area.width - 2)])
            .split(area);

        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(h_chunks[1]);

        let list = List::new(
            iter::repeat(ListItem::new(" "))
                .take(2)
                .collect::<Vec<ListItem>>(),
        )
        .highlight_symbol(">")
        .style(style);

        let input_box =
            Textbox::new().style(style.patch(Style::default().remove_modifier(Modifier::REVERSED)));

        let check_box = Checkbox::new("Checkbox!").style(style);

        frame.render_stateful_widget(list, h_chunks[0], &mut self.list_state);
        frame.render_stateful_widget(input_box, v_chunks[0], &mut self.textbox_state);
        frame.render_stateful_widget(check_box, v_chunks[1], &mut self.checkbox_state);

        if let Some(0) = self.list_state.selected() {
            let x = v_chunks[0].x + self.textbox_state.cursor_position();
            let y = v_chunks[0].y;
            frame.set_cursor(x, y);
        }
    }

    pub fn handle_input(&mut self, event: &InputEvent) -> bool {
        if match self.list_state.selected() {
            Some(0) => self.handle_input_textbox(event),
            Some(1) => self.handle_input_checkbox(event),
            _ => false,
        } {
            return true;
        }
        self.handle_input_cursor(event)
    }

    fn handle_input_textbox(&mut self, event: &InputEvent) -> bool {
        if let Some(key_code) = event.key_code {
            match key_code {
                KeyCode::Char(letter) => {
                    self.textbox_state.insert_character(letter);
                    return true;
                }
                KeyCode::Backspace => {
                    self.textbox_state.remove_character();
                    return true;
                }
                _ => {}
            }
        }

        match event.input {
            InputKind::MoveLeft(amount) => {
                self.textbox_state.move_cursor(CursorMovement::Left(amount));
                return true;
            }
            InputKind::MoveRight(amount) => {
                self.textbox_state
                    .move_cursor(CursorMovement::Right(amount));
                return true;
            }
            _ => {}
        }
        false
    }

    fn handle_input_checkbox(&mut self, event: &InputEvent) -> bool {
        match event.input {
            InputKind::Select => {
                self.checkbox_state.toggle();
                true
            }
            _ => false,
        }
    }

    fn handle_input_cursor(&mut self, event: &InputEvent) -> bool {
        match event.input {
            InputKind::MoveUp(amount) => {
                self.list_state.move_cursor(2, CursorMovement::Up(amount));
                true
            }
            InputKind::MoveDown(amount) => {
                self.list_state.move_cursor(2, CursorMovement::Down(amount));
                true
            }
            _ => false,
        }
    }
}
