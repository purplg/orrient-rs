use chrono::serde::ts_seconds;
use std::collections::HashSet;
use std::io::BufReader;
use std::io::Read;
use std::ops::Add;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;
use std::{collections::HashMap, fs::File, io::BufWriter};

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::debug;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::api::AllAccountAchievements;
use crate::api::Dailies;
use crate::api::{AccountAchievement, Achievement, AllAchievementIDs};
use crate::config::Config;

/// Controls all cached content for the app
pub struct Cache {
    path: PathBuf,
    contents: CacheContents,
    max_age: Duration,
    compression: bool,
}

/// A single cached response from the server.
// TODO Rename this or [CacheItem] to have more distinct names
#[derive(Serialize, Deserialize, Clone, Debug)]
struct CachedItem<T> {
    #[serde(with = "ts_seconds")]
    expires: DateTime<Utc>,
    inner: T,
}

impl<T> CachedItem<T> {
    pub fn new(item: T, life: Duration) -> Self
    where
        T: Serialize + DeserializeOwned,
    {
        Self {
            expires: Utc::now().add(life),
            inner: item,
        }
    }

    fn expired(&self) -> bool {
        Utc::now() > self.expires
    }
}

/// The actual cached content
#[derive(Serialize, Deserialize, Default, Debug)]
struct CacheContents {
    #[serde(skip)]
    invalid: RwLock<bool>, // TODO Should probably move this up to [Cache] somehow.
    all_achievements_ids: RwLock<Option<CachedItem<AllAchievementIDs>>>,
    achievements: RwLock<HashMap<usize, CachedItem<Achievement>>>,
    account_achievements: RwLock<Option<CachedItem<HashSet<AccountAchievement>>>>,
    dailies: RwLock<Option<CachedItem<Dailies>>>,
}

impl Cache {
    pub fn load(config: &Config) -> Self {
        let path = PathBuf::from(config.cache_path.clone());
        let contents = match Self::load_from_disk(&path, config.cache_compression) {
            Ok(contents) => contents,
            Err(err) => {
                debug!("Error loading cache from disk: {}", err);
                CacheContents::default()
            }
        };
        Self {
            path,
            contents,
            max_age: config.cache_age,
            compression: config.cache_compression,
        }
    }

    fn load_from_disk(
        path: &Path,
        use_compression: bool,
    ) -> Result<CacheContents, Box<dyn std::error::Error>> {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(&file);
        let mut contents_str = String::new();
        if use_compression {
            GzDecoder::new(reader).read_to_string(&mut contents_str)
        } else {
            reader.read_to_string(&mut contents_str)
        }?;
        Ok(serde_json::from_str::<CacheContents>(&contents_str)?)
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self
            .contents
            .invalid
            .read()
            .map(|invalid| *invalid)
            .unwrap_or_default()
        {
            debug!("Writing cache to: {:?}", self.path);
            let file = File::create(&self.path)?;
            let writer = BufWriter::new(file);
            if self.compression {
                let writer = GzEncoder::new(writer, Compression::fast());
                serde_json::to_writer(writer, &self.contents)
            } else {
                serde_json::to_writer(writer, &self.contents)
            }?;
        }
        Ok(())
    }
}

/// Controls how to commit and retrieve an item from the cache
// TODO Rename this or [CachedItem] to have more distinct names
pub trait CacheItem<P> {
    fn from_cache(cache: &Cache, key: &P) -> Option<Self>
    where
        Self: Sized;

    fn to_cache(&self, cache: &Cache)
    where
        Self: Sized;

    /// Marks this [CacheItem] as invalid so it is requested from the endpoint next time it is accessed
    fn invalidate_cache(cache: &Cache) {
        let _lock = cache
            .contents
            .invalid
            .write()
            .map(|mut invalid| *invalid = true);
    }
}

impl CacheItem<()> for AllAchievementIDs {
    fn from_cache(cache: &Cache, _: &()) -> Option<AllAchievementIDs> {
        cache
            .contents
            .all_achievements_ids
            .read()
            .map(|cache| cache.clone())
            .ok()
            .flatten()
            .filter(|cached_item| !cached_item.expired())
            .map(|cached_item| cached_item.inner)
    }

    fn to_cache(&self, cache: &Cache) {
        let _lock = cache
            .contents
            .all_achievements_ids
            .write()
            .map(|mut cached_item| {
                *cached_item = Some(CachedItem::new(self.clone(), cache.max_age))
            });
        Self::invalidate_cache(cache);
    }
}

impl CacheItem<usize> for Achievement {
    fn from_cache(cache: &Cache, id: &usize) -> Option<Achievement> {
        cache
            .contents
            .achievements
            .read()
            .map(|cache| cache.get(id).cloned())
            .ok()
            .flatten()
            .filter(|cached_item| !cached_item.expired())
            .map(|cached_item| cached_item.inner)
    }

    fn to_cache(&self, cache: &Cache) {
        let _lock =
            cache.contents.achievements.write().map(|mut cached| {
                cached.insert(self.id, CachedItem::new(self.clone(), cache.max_age))
            });
        Self::invalidate_cache(cache);
    }
}

impl CacheItem<()> for AllAccountAchievements {
    fn from_cache(cache: &Cache, _: &()) -> Option<AllAccountAchievements> {
        cache
            .contents
            .account_achievements
            .read()
            .map(|aa_cache| {
                aa_cache
                    .as_ref()
                    .filter(|aa| !aa.expired())
                    .map(|cached_item| AllAccountAchievements(cached_item.inner.clone()))
            })
            .ok()
            .flatten()
    }

    fn to_cache(&self, cache: &Cache) {
        let _lock = cache
            .contents
            .account_achievements
            .write()
            .map(|mut cached| {
                *cached = Some(CachedItem::new(self.0.clone(), Duration::minutes(1)))
            });
        Self::invalidate_cache(cache);
    }
}

impl CacheItem<usize> for AccountAchievement {
    fn from_cache(cache: &Cache, id: &usize) -> Option<AccountAchievement> {
        cache
            .contents
            .account_achievements
            .read()
            .map(|cache| {
                cache
                    .as_ref()
                    .filter(|cached_item| !cached_item.expired())
                    .map(|cached_item| {
                        cached_item
                            .inner
                            .iter()
                            .find(|a| a.id == *id)
                            .map(ToOwned::to_owned)
                    })
                    .flatten()
            })
            .ok()
            .flatten()
    }

    fn to_cache(&self, _cache: &Cache) {}
}

impl CacheItem<()> for Dailies {
    fn from_cache(cache: &Cache, _: &()) -> Option<Dailies> {
        cache
            .contents
            .dailies
            .read()
            .map(|cache| cache.clone())
            .ok()
            .flatten()
            .filter(|cached_item| !cached_item.expired())
            .map(|cached_item| cached_item.inner)
    }

    fn to_cache(&self, cache: &Cache) {
        let _lock = cache.contents.dailies.write().map(|mut cached_item| {
            *cached_item = Some(CachedItem::new(self.clone(), cache.max_age))
        });
        Self::invalidate_cache(cache);
    }
}
