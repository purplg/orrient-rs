use std::time::Duration;

use crossterm::event::Event as CrosstermEvent;
use crossterm::event::{EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use tokio::{select, sync::mpsc::UnboundedSender};

#[derive(Debug)]
pub enum InputKind {
    MoveLeft(u16),
    MoveRight(u16),
    MoveUp(u16),
    MoveDown(u16),
    Top,
    Bottom,
    Confirm,
    Select,
    Back,
    Quit,
    New,
    Search,
    SwitchTab(usize),
    Unhandled,
}

pub struct InputEvent {
    pub input: InputKind,
    pub key_code: Option<KeyCode>,
}

pub struct Input {
    tx_input: UnboundedSender<InputEvent>,
}

impl Input {
    pub fn new(tx_input: UnboundedSender<InputEvent>) -> Input {
        Input { tx_input }
    }

    pub async fn run(self) {
        let mut reader = EventStream::new();

        loop {
            let delay = tokio::time::sleep(Duration::from_millis(500)).fuse();
            let event = reader.next().fuse();
            select! {
                _ = delay => {},
                Some(Ok(event)) = event => {let _ = self.tx_input.send(self.handle(event));}
            }
        }
    }

    fn handle(&self, event: CrosstermEvent) -> InputEvent {
        if let CrosstermEvent::Key(keyevent) = event {
            let input_kind = match keyevent.code {
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => InputKind::MoveLeft(1),
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => InputKind::MoveRight(1),
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => InputKind::MoveUp(1),
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => InputKind::MoveDown(1),
                KeyCode::PageUp => InputKind::MoveUp(100),
                KeyCode::PageDown => InputKind::MoveDown(100),
                KeyCode::Esc => InputKind::Back,
                KeyCode::Home => InputKind::Top,
                KeyCode::End => InputKind::Bottom,
                KeyCode::Enter => InputKind::Confirm,
                KeyCode::Char(' ') => InputKind::Select,
                KeyCode::Char('n') => InputKind::New,
                KeyCode::Char('q') => InputKind::Quit,
                KeyCode::Char('/') => InputKind::Search,
                KeyCode::Char('1') => InputKind::SwitchTab(0),
                KeyCode::Char('2') => InputKind::SwitchTab(1),
                KeyCode::Char('3') => InputKind::SwitchTab(2),
                KeyCode::Char('4') => InputKind::SwitchTab(3),
                _ => InputKind::Unhandled,
            };

            InputEvent {
                input: input_kind,
                key_code: Some(keyevent.code),
            }
        } else {
            InputEvent {
                input: InputKind::Unhandled,
                key_code: None,
            }
        }
    }
}
