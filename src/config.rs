use std::fs;
use std::io;
use std::path::Path;

use crate::cli::Options;

use chrono::Duration;
use serde::Deserialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidYaml(serde_yaml::Error),
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub gateway: String,
    pub apikey: Option<String>,
    #[serde(default)]
    pub offline: bool,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default = "default_cache_path")]
    pub cache_path: String,
    #[serde(default = "default_cache_age", with = "duration_seconds")]
    pub cache_age: Duration,
    #[serde(default)]
    pub cache_compression: bool,
    #[serde(default = "default_starting_tab")]
    pub starting_tab: usize,
}

impl Config {
    pub fn load(options: Options) -> Result<Config> {
        let mut config =
            open_config(&options.config_path).and_then(|config| parse_config(&config))?;

        config.options_override(options);

        Ok(config)
    }

    fn options_override(&mut self, options: Options) {
        if let Some(cache_path) = options.cache_path {
            self.cache_path = cache_path;
        }

        if let Some(gateway) = options.gateway {
            self.gateway = gateway;
        }

        if let Some(cache_age) = options.cache_age {
            self.cache_age = cache_age;
        }

        if options.apikey.is_some() {
            self.apikey = options.apikey;
        }

        if options.offline {
            self.offline = options.offline;
        }

        if options.verbose {
            self.verbose = options.verbose;
        }

        if options.cache_compression {
            self.cache_compression = options.cache_compression;
        }

        if let Some(starting_tab) = options.starting_tab {
            self.starting_tab = starting_tab;
        }
    }
}

fn open_config(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(config) => Ok(config),
        Err(e) => Err(Error::Io(e)),
    }
}

fn parse_config(config: &'_ str) -> Result<Config> {
    match serde_yaml::from_str(config) {
        Ok(config) => Ok(config),
        Err(error) => Err(Error::InvalidYaml(error)),
    }
}

fn default_cache_path() -> String {
    String::from("cache.json")
}

fn default_cache_age() -> Duration {
    Duration::days(1)
}

fn default_starting_tab() -> usize {
    1
}

mod duration_seconds {
    use core::fmt;

    use chrono::Duration;
    use serde::{
        de::{Error, Visitor},
        Deserializer,
    };

    pub struct SecondsDurationVisitor;

    impl<'de> Visitor<'de> for SecondsDurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a duration in seconds")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Duration, E>
        where
            E: Error,
        {
            Ok(Duration::seconds(value))
        }

        fn visit_i8<E>(self, value: i8) -> Result<Duration, E>
        where
            E: Error,
        {
            Ok(Duration::seconds(i64::from(value)))
        }

        fn visit_i16<E>(self, value: i16) -> Result<Duration, E>
        where
            E: Error,
        {
            Ok(Duration::seconds(i64::from(value)))
        }

        fn visit_i32<E>(self, value: i32) -> Result<Duration, E>
        where
            E: Error,
        {
            Ok(Duration::seconds(i64::from(value)))
        }

        fn visit_u8<E>(self, value: u8) -> Result<Duration, E>
        where
            E: Error,
        {
            Ok(Duration::seconds(i64::from(value)))
        }

        fn visit_u16<E>(self, value: u16) -> Result<Duration, E>
        where
            E: Error,
        {
            Ok(Duration::seconds(i64::from(value)))
        }

        fn visit_u32<E>(self, value: u32) -> Result<Duration, E>
        where
            E: Error,
        {
            Ok(Duration::seconds(i64::from(value)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Duration, E>
        where
            E: Error,
        {
            if value <= i64::MAX as u64 {
                Ok(Duration::seconds(value as i64))
            } else {
                Err(E::custom(format!("u64 out of range: {}", value)))
            }
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_i64(SecondsDurationVisitor)
    }
}
