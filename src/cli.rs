use std::path::PathBuf;

use chrono::Duration;
use clap::{App, Arg};

/// Contains all the possible arguments passed from the command line.
#[derive(Debug)]
pub struct Options {
    pub config_path: PathBuf,
    pub gateway: Option<String>,
    pub apikey: Option<String>,
    pub offline: bool,
    pub verbose: bool,
    pub cache_path: Option<String>,
    pub cache_age: Option<Duration>,
    pub cache_compression: bool,
    pub starting_tab: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            config_path: PathBuf::from("config.yaml"),
            gateway: None,
            apikey: None,
            offline: false,
            verbose: false,
            cache_path: None,
            cache_age: None,
            cache_compression: false,
            starting_tab: None,
        }
    }
}

impl Options {
    // Automatically grabs, parses, and returns an Options object with all the selected user options
    pub fn new() -> Self {
        #[rustfmt::skip]
        let matches = App::new("Orrient")
            .version("0.1.0")
            .about("Keep track of dailies, achievements, crafting, etc in Guild Wars 2")
            .args(&vec![
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("Specify a config file to use")
                    .takes_value(true),
                Arg::with_name("gateway")
                    .short("g")
                    .long("gateway")
                    .value_name("URL")
                    .help("Specify a different API gateway to use")
                    .takes_value(true),
                Arg::with_name("apikey")
                    .short("k")
                    .long("apikey")
                    .value_name("API KEY")
                    .help("Specify an API key to use for requests that require it")
                    .takes_value(true),
                Arg::with_name("offline")
                    .short("o")
                    .long("offline")
                    .alias("dryrun")
                    .help("Only use the local cache. Do not query GW2 API"),
                Arg::with_name("dryrun")
                    .long("dryrun")
                    .alias("offline")
                    .help("Same thing as --offline"),
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .help("Enable verbose/debug logging"),
                Arg::with_name("cache-path")
                    .short("h")
                    .long("cache-path")
                    .help("The location to store the cache")
                    .takes_value(true),
                Arg::with_name("cache-age")
                    .short("a")
                    .long("cache-age")
                    .help("The maximum age of cached items (in seconds) before they'll be refetched")
                    .takes_value(true),
                Arg::with_name("cache-compression")
                    .short("z")
                    .long("cache-compress")
                    .help("Compress the cache file"),
                Arg::with_name("starting-tab")
                    .short("t")
                    .long("starting-tab")
                    .value_name("TAB_NUMBER")
                    .takes_value(true)
                    .help("The tab number to open on."),
            ])
            .get_matches();

        let mut options = Options::default();

        if let Some(config_path) = matches.value_of("config") {
            options.config_path = PathBuf::from(config_path.to_string());
        }

        if let Some(cache_path) = matches.value_of("cache-file") {
            options.cache_path = Some(cache_path.to_string());
        }

        if let Some(cache_age) = matches.value_of("cache-age") {
            if let Ok(cache_age) = cache_age.parse::<i64>() {
                options.cache_age = Some(Duration::seconds(cache_age));
            }
        }

        options.gateway = matches.value_of("gateway").map(ToOwned::to_owned);
        options.apikey = matches.value_of("apikey").map(ToOwned::to_owned);
        options.offline = matches.is_present("offline");
        options.verbose = matches.is_present("verbose");
        options.cache_compression = matches.is_present("cache-compression");
        options.starting_tab = matches
            .value_of("starting-tab")
            .map(|starting_tab| starting_tab.parse::<usize>().ok())
            .flatten();

        options
    }
}
