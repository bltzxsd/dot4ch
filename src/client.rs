use std::{ops::Deref, sync::Arc, time::Duration};

use crate::{error::Error, result::Result};
use reqwest::{
    header::{IF_MODIFIED_SINCE, LAST_MODIFIED, USER_AGENT},
    Client as ReqwestClient,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{Semaphore, SemaphorePermit},
    task::JoinHandle,
    time::interval,
};

/// Represents a client to perform HTTP requests with rate limiting.
///
/// The `Client` structure wraps an asynchronous HTTP client (`ReqwestClient`) and implements
/// rate limiting at 1 req/sec via a semaphore.
///
/// ## Rate Limiting
///
/// By default, the rate limiter provides one permit per second. If more requests are made
/// than allowed, the client will await until a permit becomes available before proceeding.
///
/// ## Note
///
/// `Client` supports the `Default` trait, so you can create a new instance with `Client::default()`.
#[derive(Debug)]
pub struct Client {
    /// Holds the reqwest client for accessing API
    http: ReqwestClient,
    /// Contains global rate limiter.
    limiter: RateLimit,
}

#[derive(Debug)]
pub(crate) struct RateLimit {
    pub(crate) permit: Arc<Semaphore>,
    pub(crate) replenisher: JoinHandle<()>,
}

impl RateLimit {
    pub async fn acquire(&self) -> Result<SemaphorePermit> {
        self.permit.acquire().await.map_err(Into::into)
    }
}

impl Client {
    /// Creates a new instance of `Client`.
    ///
    /// This function initializes the HTTP client and configures the rate-limiting logic.
    ///
    /// # Internals
    ///
    /// This function spawns a background task to add permits to the semaphore at rate of
    /// +1 permit per second
    pub fn new() -> Client {
        let http = ReqwestClient::new();

        let permit = Arc::new(Semaphore::new(0));
        let clone = permit.clone();

        let replenisher = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if clone.available_permits() == 0 {
                    clone.add_permits(1);
                }
            }
        });

        let limiter = RateLimit {
            permit,
            replenisher,
        };

        Client { http, limiter }
    }

    pub(crate) async fn fetch_json<T>(
        &self,
        url: &str,
        last_modified: Option<&str>,
    ) -> Result<Reply<T>>
    where
        T: for<'a> Deserialize<'a> + Serialize,
    {
        use reqwest::StatusCode;

        let permit = self.limiter.acquire().await?;
        let response = {
            let mut builder = self.http.get(url).header(USER_AGENT, "Dot4chClient/1.0");
            if let Some(time) = last_modified {
                builder = builder.header(IF_MODIFIED_SINCE, time);
            }
            log::info!("request for {} dispatched", url);
            builder.send().await?
        };
        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|x| x.to_str().ok())
            .map(ToString::to_string);

        // reduce the permit count
        permit.forget();

        log::info!("response: {:#?}", &response);
        log::info!("response status: {}", &response.status());

        let inner = match response.status() {
            StatusCode::OK => response.json::<T>().await.map_err(Into::into),
            StatusCode::NOT_MODIFIED => Err(Error::NotModified),
            code => Err(Error::UnexpectedStatus(code)),
        };

        Ok(Reply {
            inner,
            last_modified,
        })
    }
}

impl Drop for RateLimit {
    fn drop(&mut self) {
        self.replenisher.abort();
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) struct Reply<T: Serialize + for<'a> Deserialize<'a>> {
    pub(crate) inner: Result<T>,
    pub(crate) last_modified: Option<String>,
}

impl<T: Serialize + for<'a> Deserialize<'a>> Deref for Reply<T> {
    type Target = Result<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
