use std::cmp::min;

use tui::widgets::ListState;

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
                    if selected > amount as usize {
                        selected - amount as usize
                    } else {
                        0
                    }
                }
                CursorMovement::Down(amount) => selected + amount as usize,
                _ => selected,
            };
            selected = min(selected, total_items - 1);
            Some(selected)
        });
    }
}

#[derive(Debug)]
pub enum CursorMovement {
    Left(u16),
    Right(u16),
    Up(u16),
    Down(u16),
    None,
}
