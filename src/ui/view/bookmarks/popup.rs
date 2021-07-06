use std::iter;

use crossterm::event::KeyCode;
use tui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{
    bookmarks::Bookmark,
    input::{InputEvent, InputKind},
    ui::widget::{
        list_selection::{CursorMovement, ListSelection},
        text_box::{Textbox, TextboxState},
    },
};

pub struct CustomBookmarkPopupState {
    name_textbox_state: TextboxState,
    link_textbox_state: TextboxState,
    list_state: ListState,
}

impl Default for CustomBookmarkPopupState {
    fn default() -> Self {
        let mut popup = Self {
            name_textbox_state: TextboxState::default(),
            link_textbox_state: TextboxState::default(),
            list_state: ListState::default(),
        };
        popup.list_state.select(Some(0));
        popup
    }
}

impl CustomBookmarkPopupState {
    pub fn cancel(&mut self) {
        self.name_textbox_state = TextboxState::default();
        self.link_textbox_state = TextboxState::default();
        self.list_state = ListState::default();
        self.list_state.select(Some(0));
    }

    pub fn finish(&mut self) -> Bookmark {
        let bookmark = Bookmark {
            kind: crate::bookmarks::BookmarkKind::Waypoint,
            name: self.name_textbox_state.take(),
            link: self.link_textbox_state.take(),
        };
        self.cancel();
        bookmark
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
            .title("Add bookmark")
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

        let name_input_box =
            Textbox::new().style(style.patch(Style::default().remove_modifier(Modifier::REVERSED)));

        let link_input_box =
            Textbox::new().style(style.patch(Style::default().remove_modifier(Modifier::REVERSED)));

        frame.render_stateful_widget(list, h_chunks[0], &mut self.list_state);
        frame.render_stateful_widget(name_input_box, v_chunks[0], &mut self.name_textbox_state);
        frame.render_stateful_widget(link_input_box, v_chunks[1], &mut self.link_textbox_state);

        match self.list_state.selected() {
            Some(0) => {
                let x = v_chunks[0].x + self.name_textbox_state.cursor_position();
                let y = v_chunks[0].y;
                frame.set_cursor(x, y);
            }
            Some(1) => {
                let x = v_chunks[1].x + self.link_textbox_state.cursor_position();
                let y = v_chunks[1].y;
                frame.set_cursor(x, y);
            }
            _ => {}
        }
    }

    pub fn handle_input(&mut self, event: &InputEvent) -> bool {
        if match self.list_state.selected() {
            Some(0) => self.handle_input_name_textbox(event),
            Some(1) => self.handle_input_link_textbox(event),
            _ => false,
        } {
            return true;
        }
        self.handle_input_cursor(event)
    }

    fn handle_input_name_textbox(&mut self, event: &InputEvent) -> bool {
        if let Some(key_code) = event.key_code {
            match key_code {
                KeyCode::Char(letter) => {
                    self.name_textbox_state.insert_character(letter);
                    return true;
                }
                KeyCode::Backspace => {
                    self.name_textbox_state.remove_character();
                    return true;
                }
                _ => {}
            }
        }

        match event.input {
            InputKind::MoveLeft(amount) => {
                self.name_textbox_state
                    .move_cursor(CursorMovement::Left(amount));
                return true;
            }
            InputKind::MoveRight(amount) => {
                self.name_textbox_state
                    .move_cursor(CursorMovement::Right(amount));
                return true;
            }
            _ => {}
        }
        false
    }

    fn handle_input_link_textbox(&mut self, event: &InputEvent) -> bool {
        if let Some(key_code) = event.key_code {
            match key_code {
                KeyCode::Char(letter) => {
                    self.link_textbox_state.insert_character(letter);
                    return true;
                }
                KeyCode::Backspace => {
                    self.link_textbox_state.remove_character();
                    return true;
                }
                _ => {}
            }
        }

        match event.input {
            InputKind::MoveLeft(amount) => {
                self.link_textbox_state
                    .move_cursor(CursorMovement::Left(amount));
                return true;
            }
            InputKind::MoveRight(amount) => {
                self.link_textbox_state
                    .move_cursor(CursorMovement::Right(amount));
                return true;
            }
            _ => {}
        }
        false
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
