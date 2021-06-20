use futures::stream::StreamExt;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::sync::mpsc::UnboundedSender;

use crate::events::{Event, StateEvent};

pub(crate) async fn handle_signals(signals: Signals, tx_event: UnboundedSender<Event>) {
    let handle = signals.handle();
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGINT | SIGQUIT | SIGTERM => {
                let _ = tx_event.send(Event::State(StateEvent::Quit));
            }
            _ => unreachable!(),
        }
    }
    handle.close();
}
