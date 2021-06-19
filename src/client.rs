use std::{fmt::Debug, hash::Hash, sync::Arc};

use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;

use crate::{api::endpoints::Endpoint, cache::{Cache, CacheItem}, config::Config};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingApiKey,
    Request(reqwest::Error),
    Json(serde_json::Error),
}

/// Tries to make requests to the [Cache] before it reachs out to the gateway
pub struct CachedClient {
    client: GW2Client,
    cache: Arc<Cache>,
}

impl CachedClient {
    pub fn new(config: Config) -> Result<Self> {
        let client = GW2Client::new(&config);
        let cache = Arc::new(Cache::load(&config));
        Ok(CachedClient { client, cache })
    }

    /// Tell cache to commit to disk
    // TODO this doesn't belong here...?
    pub fn write_cache(&self) {
        let _ = self.cache.write();
    }

    /// Make a cached request for a single [Endpoint]
    pub async fn request<E>(&self) -> Result<E>
    where
        E: Endpoint<()> + CacheItem<()> + DeserializeOwned + Clone + Debug,
    {
        match E::from_cache(&self.cache, &()) {
            Some(cached) => Ok(cached),
            None => {
                let response = self.client.request::<E>().await?;
                response.to_cache(&self.cache);
                Ok(response)
            }
        }
    }

    /// Make a cached request for a list of [Endpoint]'s from a list of parameters
    pub async fn request_many<E, P>(&self, params: &[P]) -> Result<Vec<E>>
    where
        E: Endpoint<P> + CacheItem<P> + DeserializeOwned + Debug + Send,
        P: Hash + Eq + Debug,
    {
        // Sort out which items are already cached and which need to be fetched from the gateway
        let mut request_items: Vec<&P> = vec![];
        let cached_items: Vec<E> = params
            .iter()
            .map(|param| {
                E::from_cache(&self.cache, &param).or_else(|| {
                    request_items.push(param);
                    None
                })
            })
            .flatten()
            .collect();

        // If everything  was cached then we don't need to continue to make the gateway request
        if request_items.is_empty() {
            return Ok(cached_items);
        }

        // Something wasn't in cache so make a gateway request
        match self.client.request_with_params::<E, P>(request_items).await {
            Ok(remote_items) => {
                for item in &remote_items {
                    item.to_cache(&self.cache);
                }
                Ok(cached_items
                    .into_iter()
                    .chain(remote_items.into_iter())
                    .collect::<Vec<E>>())
            }
            Err(e) => Err(e),
        }
    }
}

/// A client for making requests to a Guild Wars 2 API gateway
pub struct GW2Client {
    client: reqwest::Client,
    gateway: String,
    apikey: Option<String>,
}

impl GW2Client {
    pub fn new(config: &Config) -> GW2Client {
        GW2Client {
            client: reqwest::Client::new(),
            gateway: config.gateway.clone(),
            apikey: config.apikey.to_owned(),
        }
    }

    /// Make an uncached request for a single [Endpoint]
    pub async fn request<E>(&self) -> Result<E>
    where
        E: Endpoint<()> + CacheItem<()> + DeserializeOwned,
    {
        let request_builder = self
            .client
            .get(format!("{}/{}", self.gateway, E::get_path(vec![&()])))
            .headers(self.get_headers::<E, ()>()?);

        let response: reqwest::Response = request_builder.send().await.map_err(|err| Error::Request(err))?;
        match response.text().await {
            Ok(text) => match serde_json::from_str::<E>(text.as_str()) {
                Ok(endpoint) => Ok(endpoint),
                Err(err) => Err(Error::Json(err)),
            },
            Err(err) => Err(Error::Request(err)),
        }
    }

    /// Make an uncached request for a list of [Endpoint]'s from a list of parameters
    pub async fn request_with_params<E, P>(&self, params: Vec<&P>) -> Result<Vec<E>>
    where
        E: Endpoint<P> + DeserializeOwned,
    {
        let request_builder = self
            .client
            .get(format!("{}/{}", self.gateway, E::get_path(params)))
            .headers(self.get_headers::<E, P>()?);

        let response: reqwest::Response = request_builder.send().await.map_err(|err| Error::Request(err))?;
        match response.text().await {
            Ok(text) => match serde_json::from_str::<Vec<E>>(text.as_str()) {
                Ok(endpoint) => Ok(endpoint),
                Err(err) => Err(Error::Json(err)),
            },
            Err(err) => Err(Error::Request(err)),
        }
    }

    fn get_headers<E, P>(&self) -> Result<HeaderMap>
    where
        E: Endpoint<P>,
    {
        let mut headermap = HeaderMap::new();

        // This header is needed to use the new schema for the Dailies endpoint
        headermap.insert(
            "X-Schema-Version",
            "2019-05-16T00:00:00.000Z".parse().unwrap(),
        );

        // Place the API key in the headers only if it is necessary
        if E::AUTHENTICATED {
            if let Some(apikey) = &self.apikey {
                headermap.insert(
                    "Authorization",
                    format!("Bearer {}", apikey).parse().unwrap(),
                );
            } else {
                return Err(Error::MissingApiKey);
            }
        }

        Ok(headermap)
    }
}
