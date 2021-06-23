use std::cmp::min;

use tui::widgets::ListState;

use crate::events::CursorMovement;

pub trait ListSelection {
    fn move_cursor(&mut self, total_items: usize, movement: CursorMovement);
}

impl ListSelection for ListState {
    fn move_cursor(&mut self, total_items: usize, movement: CursorMovement) {
        self.select(if total_items == 0 {
            None
        } else {
            let mut selected = self.selected().unwrap_or_default();
            selected = match movement {
                CursorMovement::Up(amount) => {
                    if selected > amount {
                        selected - amount
                    } else {
                        0
                    }
                }
                CursorMovement::Down(amount) => selected + amount,
                _ => selected,
            };
            selected = min(selected, total_items - 1);
            Some(selected)
        });
    }
}
