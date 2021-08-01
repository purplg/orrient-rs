use std::{io::Stdout, time::Duration};

use futures::FutureExt;
use orrient::events::Event;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::input::InputEvent;

use super::View;

pub struct StatusView {
    message: String,
    tx_event: UnboundedSender<Event>,
    status_timeout_handle: Option<JoinHandle<()>>,
}

impl StatusView {
    pub fn new(tx_event: UnboundedSender<Event>) -> Self {
        StatusView {
            message: String::default(),
            tx_event,
            status_timeout_handle: None,
        }
    }
}

impl StatusView {
    fn start_timeout(&mut self) {
        if let Some(handle) = &self.status_timeout_handle {
            handle.abort();
        }
        self.status_timeout_handle = Some(status_timeout(self.tx_event.clone()));
    }
}

impl View for StatusView {
    fn name(&self) -> &'static str {
        "Status"
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        frame.render_widget(
            Paragraph::new(self.message.as_str()).block(Block::default().borders(Borders::TOP)),
            area,
        );
    }

    fn handle_input(&mut self, _: &InputEvent) -> bool {
        false
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::StatusMessage(message) => {
                self.message = message.clone();
                self.start_timeout();
            }
            Event::ClearStatusMessage => {
                self.message.clear();
            }
            _ => {}
        }
    }
}

fn status_timeout(tx_event: UnboundedSender<Event>) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).fuse().await;
        let _ = tx_event.send(Event::ClearStatusMessage);
    })
}
