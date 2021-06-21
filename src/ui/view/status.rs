use std::time::Duration;

use crate::events::{InputEvent, ViewEvent};
use futures::FutureExt;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};
use tui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::View;

pub struct StatusView {
    message: String,
    tx_view_event: UnboundedSender<ViewEvent>,
    status_timeout_handle: Option<JoinHandle<()>>,
}

impl StatusView {
    pub fn new(tx_view_event: UnboundedSender<ViewEvent>) -> Self {
        StatusView {
            message: String::default(),
            tx_view_event,
            status_timeout_handle: None,
        }
    }
}

impl StatusView {
    fn start_timeout(&mut self) {
        if let Some(handle) = &self.status_timeout_handle {
            handle.abort();
        }
        self.status_timeout_handle = Some(status_timeout(self.tx_view_event.clone()));
    }
}

impl View for StatusView {
    fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        frame.render_widget(
            Paragraph::new(self.message.as_str()).block(Block::default().borders(Borders::TOP)),
            area,
        );
    }

    fn handle_input_event(&mut self, _: &InputEvent) -> bool {
        false
    }

    fn handle_view_event(&mut self, view_event: &ViewEvent) {
        if let ViewEvent::UpdateStatus(message) = view_event {
            self.message = message.clone();
            self.start_timeout();
        }
    }
}

fn status_timeout(tx_view_event: UnboundedSender<ViewEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).fuse().await;
        let _ = tx_view_event.send(ViewEvent::UpdateStatus("".to_string()));
    })
}
