use std::num::NonZeroU32;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

const USER_AGENT: &str =
    "wiki-lore-crawler/0.1 (https://github.com/anomalyco/lorebook-builder)";

/// MediaWiki API client with rate limiting and automatic continue-token
/// pagination.
pub struct ApiClient {
    http: Client,
    base: String,
    limiter: Limiter,
}

impl ApiClient {
    pub fn new(api_base: impl Into<String>, req_per_sec: u32) -> Result<Self> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(45))
            .gzip(true)
            .build()
            .context("building http client")?;
        let rps = NonZeroU32::new(req_per_sec.max(1)).unwrap();
        let q = Quota::per_second(rps);
        let limiter = RateLimiter::direct(q);
        Ok(Self {
            http,
            base: api_base.into(),
            limiter,
        })
    }

    pub fn base(&self) -> &str { &self.base }

    /// Build a single API URL.
    pub fn url(&self, params: &[(&str, String)]) -> String {
        let mut s = format!("{}?", self.base);
        let mut first = true;
        for (k, v) in params {
            if !first { s.push('&'); }
            s.push_str(k);
            s.push('=');
            s.push_str(&url_encode(v));
            first = false;
        }
        s.push_str("&format=json");
        s
    }

    /// GET a JSON value. Honors rate limiting.
    pub async fn get_json(&self, url: &str) -> Result<Value> {
        self.limiter.until_ready().await;
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 0..3 {
            let r = self.http.get(url).send().await;
            match r {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let bytes = resp.bytes().await?;
                        let v: Value = serde_json::from_slice(&bytes)
                            .with_context(|| format!("parsing JSON from {}", url))?;
                        return Ok(v);
                    } else if status.as_u16() == 429 || status.is_server_error() {
                        let backoff = Duration::from_millis(200 * (1u64 << attempt));
                        tokio::time::sleep(backoff).await;
                        last_err = Some(anyhow!("HTTP {} from {}", status, url));
                        continue;
                    } else {
                        let body = resp.text().await.unwrap_or_default();
                        return Err(anyhow!("HTTP {} from {}: {}", status, url, body));
                    }
                }
                Err(e) => {
                    let backoff = Duration::from_millis(200 * (1u64 << attempt));
                    tokio::time::sleep(backoff).await;
                    last_err = Some(e.into());
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow!("request failed after retries: {}", url)))
    }

    /// Helper: fetch a single API action and decode to a typed value.
    pub async fn call<T: DeserializeOwned>(&self, params: &[(&str, String)]) -> Result<T> {
        let url = self.url(params);
        let v = self.get_json(&url).await?;
        Ok(serde_json::from_value(v)?)
    }
}

pub fn url_encode(s: &str) -> String {
    percent_encoding::utf8_percent_encode(
        s,
        percent_encoding::NON_ALPHANUMERIC,
    ).to_string()
}
