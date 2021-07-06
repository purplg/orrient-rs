mod popup;

use std::rc::Rc;

use copypasta::{ClipboardContext, ClipboardProvider};
use log::debug;
use tokio::sync::mpsc::UnboundedSender;
use tui::{
    layout::Rect,
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::{
    bookmarks::Bookmark,
    events::Event,
    input::{InputEvent, InputKind},
    state::AppState,
    ui::widget::list_selection::{CursorMovement, ListSelection},
};

use self::popup::CustomBookmarkPopupState;

use super::View;

pub struct BookmarksView {
    app_state: Rc<AppState>,
    tx_state: UnboundedSender<Event>,
    clipboard_ctx: Option<ClipboardContext>,
    bookmarks: Vec<Bookmark>,
    list_state: ListState,
    add_popup: CustomBookmarkPopupState,
    inserting: bool,
}

impl BookmarksView {
    pub fn new(app_state: Rc<AppState>, tx_state: UnboundedSender<Event>) -> Self {
        let clipboard_ctx = match ClipboardContext::new() {
            Ok(ctx) => Some(ctx),
            Err(err) => {
                debug!("Could not load clipboard context: {}.", err);
                None
            }
        };
        let bookmarks = app_state.bookmarks().into_iter().collect::<Vec<Bookmark>>();
        let mut list_state = ListState::default();
        list_state.move_cursor(bookmarks.len(), CursorMovement::None);
        Self {
            app_state,
            tx_state,
            clipboard_ctx,
            bookmarks,
            list_state,
            add_popup: CustomBookmarkPopupState::default(),
            inserting: false,
        }
    }
}

impl View for BookmarksView {
    fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let list = List::new(
            self.bookmarks
                .iter()
                .map(|bookmark| ListItem::new(bookmark.name.as_str()))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_symbol(">>");
        frame.render_stateful_widget(list, area, &mut self.list_state);

        if self.inserting {
            self.add_popup.draw(frame, area);
        }
    }

    fn handle_input_event(&mut self, event: &InputEvent) -> bool {
        if !self.inserting || !self.add_popup.handle_input(event) {
            match event.input {
                InputKind::MoveUp(amount) => {
                    self.list_state
                        .move_cursor(self.bookmarks.len(), CursorMovement::Up(amount));
                    true
                }
                InputKind::MoveDown(amount) => {
                    self.list_state
                        .move_cursor(self.bookmarks.len(), CursorMovement::Down(amount));
                    true
                }
                InputKind::New => {
                    self.inserting = true;
                    true
                }
                InputKind::Back => {
                    if self.inserting {
                        self.inserting = false;
                        self.add_popup.cancel();
                    }
                    true
                }
                InputKind::Delete => {
                    if let Some(selected) = self.list_state.selected() {
                        self.app_state
                            .remove_bookmark(self.bookmarks.remove(selected));
                        self.list_state
                            .move_cursor(self.bookmarks.len(), CursorMovement::None);
                    }
                    true
                }
                InputKind::Confirm => {
                    if self.inserting {
                        self.inserting = false;
                        let bookmark = self.add_popup.finish();
                        self.app_state.add_bookmark(bookmark.clone());
                        self.bookmarks.push(bookmark);
                    } else {
                        if let Some(clipboard) = self.clipboard_ctx.as_mut() {
                            if let Some(selected) = self.list_state.selected() {
                                if let Some(bookmark) = self.bookmarks.get(selected) {
                                    let _ = clipboard.set_contents(bookmark.link.clone());
                                    let _ = self.tx_state.send(Event::StatusMessage(format!(
                                        "Copied content of '{}' to clipboard.",
                                        bookmark.name
                                    )));
                                }
                            }
                        }
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn handle_event(&mut self, _: &crate::events::Event) {}
}