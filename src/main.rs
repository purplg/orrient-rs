mod api;
mod cache;
mod cli;
mod client;
mod config;
mod events;
mod fetch;
mod input;
mod log;
mod state;
mod tracks;
mod ui;

use std::{fmt::Debug, rc::Rc, sync::Arc};

use ::log::debug;
use events::ViewEvent;
use state::AppState;
use tokio::{
    select,
    sync::mpsc,
};

use crate::{
    cli::Options,
    client::CachedClient,
    config::Config,
    events::{Event, EventLoop, StateEvent},
    fetch::Fetch,
    log::setup_logger,
    tracks::{Reader, Track},
    ui::UI,
};

#[macro_use]
extern crate serde_derive;
extern crate fern;
extern crate getopts;

type Result = std::result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    Logger(fern::InitError),
    Client(client::Error),
}

#[tokio::main]
pub async fn main() -> Result {
    let options = Options::new();
    let config: Config = Config::load(options).map_err(|err| Error::Config(err))?;

    setup_logger(&config).map_err(|err| Error::Logger(err))?;
    debug!("{:?}", config);

    let (tx_event, rx_event) = mpsc::unbounded_channel::<Event>();
    let (tx_view_event, rx_view_event) = mpsc::unbounded_channel::<ViewEvent>();

    let (tracks, tracks_writer) = Track::load("tracks.json");
    let _ = tx_event.send(Event::State(StateEvent::LoadTracks(tracks)));

    let app_state = Rc::new(AppState::new(&config));

    let client = Arc::new(CachedClient::new(config).map_err(|err| Error::Client(err))?);

    let event_loop = EventLoop::new(
        app_state.clone(),
        tx_event.clone(),
        rx_event,
        tx_view_event,
        tracks_writer,
    );
    let fetch = Fetch::new(client, tx_event.clone());
    let ui = UI::new(app_state.clone(), tx_event, rx_view_event);

    select! {
        _ = event_loop.run() => {}
        _ = fetch.run(60) => {}
        _ = ui.run() => {}
    }
    Ok(())
}
