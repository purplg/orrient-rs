mod api;
mod cache;
mod cli;
mod client;
mod config;
mod events;
mod fetch;
mod input;
mod log;
mod signals;
mod state;
mod tracks;
mod ui;

use std::{fmt::Debug, rc::Rc, sync::Arc};

use ::log::debug;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use state::AppState;
use tokio::{select, sync::mpsc};

use crate::{
    cli::{config_path, Options},
    client::CachedClient,
    config::Config,
    events::Event,
    fetch::Fetch,
    log::setup_logger,
    signals::handle_signals,
    ui::UI,
};

#[macro_use]
extern crate serde_derive;
extern crate fern;
extern crate getopts;
extern crate signal_hook;

type Result = std::result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    Logger(fern::InitError),
    Client(client::Error),
    Signal(std::io::Error),
}

#[tokio::main]
pub async fn main() -> Result {
    let options = Options::new();
    let config = Config::load(options).map_err(Error::Config);
    if let Err(Error::Config(config::Error::MissingApiKey)) = config {
        print!("You must provide an API key in the config file");
        if let Some(config_path) = config_path() {
            print!(": {}", config_path.to_string_lossy());
        }
        println!();
    }
    let config = config?;

    setup_logger(&config).map_err(Error::Logger)?;
    debug!("{:?}", config);

    let (tx_event, rx_event) = mpsc::unbounded_channel::<Event>();

    let app_state = Rc::new(AppState::load("state.json"));

    let client = Arc::new(CachedClient::new(config).map_err(Error::Client)?);

    let fetch = Fetch::new(client, tx_event.clone());
    let ui = UI::new(app_state.clone(), tx_event.clone(), rx_event);

    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT]).map_err(Error::Signal)?;

    select! {
        _ = handle_signals(signals, tx_event) => {}
        _ = fetch.run(60) => {}
        _ = ui.run() => {}
    }
    Ok(())
}
