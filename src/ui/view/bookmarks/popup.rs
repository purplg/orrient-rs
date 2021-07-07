use std::{cell::Cell, iter};

use tokio::sync::mpsc::UnboundedSender;
use tui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{
    bookmarks::Bookmark,
    events::Event,
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
    active: Cell<bool>,
    tx_event: UnboundedSender<Event>,
}

impl CustomBookmarkPopupState {
    pub fn new(tx_event: UnboundedSender<Event>) -> Self {
        let mut popup = Self {
            name_textbox_state: TextboxState::default(),
            link_textbox_state: TextboxState::default(),
            list_state: ListState::default(),
            active: Cell::new(false),
            tx_event,
        };
        popup.list_state.select(Some(0));
        popup
    }

    pub fn active(&self, active: bool) {
        self.active.set(active);
    }

    fn reset(&mut self) {
        self.name_textbox_state = TextboxState::default();
        self.link_textbox_state = TextboxState::default();
        self.list_state = ListState::default();
        self.list_state.select(Some(0));
    }

    fn finish(&mut self) -> Bookmark {
        let bookmark = Bookmark {
            kind: crate::bookmarks::BookmarkKind::Waypoint,
            name: self.name_textbox_state.take(),
            link: self.link_textbox_state.take(),
        };
        self.reset();
        bookmark
    }

    pub fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        if !self.active.get() {
            return;
        }

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
        if !self.active.get() {
            return false;
        }

        if match self.list_state.selected() {
            Some(0) => self.name_textbox_state.handle_input(event),
            Some(1) => self.link_textbox_state.handle_input(event),
            _ => false,
        } {
            return true;
        }

        match event.input {
            InputKind::Back => {
                self.active(false);
                true
            }
            InputKind::Confirm => {
                self.active(false);
                let bookmark = self.finish();
                let _ = self.tx_event.send(Event::AddBookmark(bookmark));
                true
            }
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
