mod input;
mod signals;
mod ui;

use std::{fmt::Debug, rc::Rc};

use ::log::debug;
use orrient::{
    cli::{config_path, Options},
    client::{self, CachedClient},
    config::{self, Config},
    events::Event,
    fetch::Fetch,
    log::setup_logger,
    state::AppState,
};
use signals::handle_signals;
use ui::UI;

use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::SignalsInfo;
use tokio::{select, sync::mpsc};

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

    let app_state = Rc::new(AppState::load("state.ron"));
    let ui = UI::new(&config, app_state.clone(), tx_event.clone(), rx_event);

    let client = CachedClient::new(config).map_err(Error::Client)?;
    let fetch = Fetch::new(client, tx_event.clone());

    let signals = SignalsInfo::new(&[SIGTERM, SIGINT, SIGQUIT]).map_err(Error::Signal)?;

    select! {
        _ = handle_signals(signals, tx_event) => {}
        _ = fetch.run(60) => {}
        _ = ui.run() => {}
    }
    Ok(())
}
