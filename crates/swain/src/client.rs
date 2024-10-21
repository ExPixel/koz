use std::sync::Arc;

use parking_lot::RwLock;
use reqwest::{Client, Method as RequestMethod, Url};
use serde::de::DeserializeOwned;

use crate::{
    error::{Error, Result},
    rate_limit::{RateLimitedRequest, RateLimiter},
    MethodId, Subdomain,
};

pub struct RiotHttpClient {
    client: Client,
    api_key: RwLock<String>,
    rate_limiter: Arc<RateLimiter>,
}

impl RiotHttpClient {
    pub(crate) fn new(client: Client, api_key: String) -> Self {
        let rate_limiter = Arc::new(RateLimiter::default());
        Self {
            client,
            rate_limiter,
            api_key: RwLock::new(api_key),
        }
    }

    pub fn set_api_key(&self, api_key: &str) {
        let mut locked = self.api_key.write();
        locked.clear();
        locked.push_str(api_key);
    }

    pub fn api_key(&self) -> String {
        self.api_key.read().clone()
    }

    fn request(
        &self,
        method: RequestMethod,
        path: &str,
        method_id: MethodId,
        subdomain: Subdomain,
    ) -> RiotRequestBuilder {
        let rate_limiter = self.rate_limiter.clone();

        let normalized_path = path.strip_prefix('/').unwrap_or(path);
        let url = Url::parse(&format!(
            "https://{}/{}",
            subdomain.domain(),
            normalized_path
        ))
        .expect("invalid url");
        let request = self.client.request(method, url);
        RiotRequestBuilder {
            inner: request,
            subdomain,
            method_id,
            rate_limiter,
        }
    }

    pub(crate) fn get(
        &self,
        path: &str,
        method_id: MethodId,
        subdomain: Subdomain,
    ) -> RiotRequestBuilder {
        self.request(RequestMethod::GET, path, method_id, subdomain)
    }
}

pub struct RiotRequestBuilder {
    inner: reqwest::RequestBuilder,
    subdomain: Subdomain,
    method_id: MethodId,
    rate_limiter: Arc<RateLimiter>,
}

impl RiotRequestBuilder {
    pub async fn send_riot(self) -> Result<reqwest::Response> {
        self.rate_limiter
            .send(RateLimitedRequest::new(
                self.subdomain,
                self.method_id,
                self.inner,
            ))
            .await
    }

    pub async fn send_riot_json<T: DeserializeOwned>(self) -> Result<T> {
        let response = self.send_riot().await?;
        let response_text = response.text().await.map_err(Error::ResponseContent)?;
        let object =
            serde_json::from_str(response_text.as_str()).map_err(|err| Error::Deserialize {
                err,
                source: response_text,
            })?;
        Ok(object)
    }
}

impl std::ops::Deref for RiotRequestBuilder {
    type Target = reqwest::RequestBuilder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
