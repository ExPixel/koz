use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use ahash::AHashMap;
use reqwest::Response;
use reqwest::{RequestBuilder, StatusCode};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::OwnedMutexGuard as AsyncOwnedMutexGuard;
use tokio::sync::RwLock as AsyncRwLock;

use crate::{MethodId, Subdomain};

pub struct RateLimiter {
    subdomain_limits: AHashMap<Subdomain, SubdomainRateLimiter>,
}

impl RateLimiter {
    pub(crate) async fn send(&self, request: RateLimitedRequest) -> Result<Response> {
        let limits = self
            .subdomain_limits
            .get(&request.subdomain)
            .expect("subdomain not found");
        limits.send(request.method, request.inner).await
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        let mut subdomain_limits = AHashMap::new();
        for &subdomain in Subdomain::VARIANTS {
            subdomain_limits.insert(subdomain, SubdomainRateLimiter::default());
        }
        Self { subdomain_limits }
    }
}

struct SubdomainRateLimiter {
    retry_after: AsyncRwLock<Instant>,
    buckets: Arc<AsyncMutex<RateLimitBuckets>>,
    method_buckets: AsyncRwLock<AHashMap<MethodId, Arc<AsyncMutex<RateLimitBuckets>>>>,
}

impl Default for SubdomainRateLimiter {
    fn default() -> Self {
        Self {
            retry_after: AsyncRwLock::new(Instant::now()),
            buckets: Arc::new(AsyncMutex::new(RateLimitBuckets::default())),
            method_buckets: AsyncRwLock::new(AHashMap::new()),
        }
    }
}

impl SubdomainRateLimiter {
    async fn send(&self, method: MethodId, request: RequestBuilder) -> Result<Response> {
        let maybe_method_lock = self.method_acquire(method).await;
        let maybe_subdomain_lock = self.subdomain_acquire().await;

        let retry_after = { *self.retry_after.read().await };
        if retry_after > Instant::now() {
            tokio::time::sleep_until(retry_after.into()).await;
        }

        let response = request.send().await.map_err(Error::RequestSend)?;
        drop(maybe_subdomain_lock);
        drop(maybe_method_lock);

        if response.status().is_success() {
            Ok(response)
        } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let maybe_retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            if let Some(retry_after) = maybe_retry_after {
                let mut retry_after_lock = self.retry_after.write().await;
                *retry_after_lock =
                    retry_after_lock.max(Instant::now() + Duration::from_secs(retry_after));
            }
            Err(Error::TooManyRequests)
        } else {
            Err(Error::api_error(response).await)
        }
    }

    async fn subdomain_acquire(&self) -> Option<AsyncOwnedMutexGuard<RateLimitBuckets>> {
        let mut buckets = self.buckets.clone().lock_owned().await;
        if buckets.is_empty() {
            Some(buckets)
        } else {
            buckets.acquire().await;
            None
        }
    }

    async fn method_acquire(
        &self,
        method: MethodId,
    ) -> Option<AsyncOwnedMutexGuard<RateLimitBuckets>> {
        {
            let method_buckets = self.method_buckets.read().await;

            if let Some(buckets) = method_buckets.get(&method) {
                let mut buckets = buckets.lock().await;
                buckets.acquire().await;
                return None;
            }
        }

        let mut method_buckets = self.method_buckets.write().await;

        match method_buckets.entry(method) {
            Entry::Occupied(entry) => {
                let mut buckets = entry.get().clone().lock_owned().await;
                if !buckets.is_empty() {
                    buckets.acquire().await;
                    None
                } else {
                    Some(buckets)
                }
            }
            Entry::Vacant(entry) => {
                let buckets = Arc::new(AsyncMutex::new(RateLimitBuckets::default()));
                Some(entry.insert(buckets).clone().lock_owned().await)
            }
        }
    }
}

#[derive(Default)]
struct RateLimitBuckets {
    buckets: AHashMap<u16, RateLimitBucket>,
}

impl RateLimitBuckets {
    async fn acquire(&mut self) {
        if let Err(wait_until) = self.check() {
            tokio::time::sleep_until(wait_until.into()).await;
        }
    }
}

impl RateLimitBuckets {
    fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }

    fn check(&mut self) -> Result<(), Instant> {
        let mut maybe_wait_until = None;
        let now = Instant::now();
        for bucket in self.buckets.values_mut() {
            let Err(next_allowed_at) = bucket.check_arrival(now) else {
                continue;
            };
            maybe_wait_until = maybe_wait_until.max(Some(next_allowed_at));
        }

        if let Some(wait_until) = maybe_wait_until {
            Err(wait_until)
        } else {
            Ok(())
        }
    }
}

pub struct RateLimitBucket {
    /// Theoretical arrival time of the next event.
    tat: Instant,
    period: Duration,
    rate: u32,
    burst: u32,
    max_burst: u32,
}

impl RateLimitBucket {
    pub fn with_arrival(
        initial_arrival_time: Instant,
        period: Duration,
        rate: u32,
        max_burst: u32,
    ) -> Self {
        let emission_interval = period / rate;
        let tat = initial_arrival_time + emission_interval;

        Self {
            tat,
            period,
            rate,
            burst: max_burst.saturating_sub(1),
            max_burst,
        }
    }

    fn emission_interval(&self) -> Duration {
        self.period / self.rate
    }

    fn delay_tolerance(&self) -> Duration {
        self.emission_interval() * self.burst
    }

    pub fn check_now(&mut self) -> Result<(), Instant> {
        self.check_arrival(Instant::now())
    }

    pub fn check_arrival(&mut self, arrival: Instant) -> Result<(), Instant> {
        let tat = self.tat.max(arrival);
        let allowed_at = tat - self.delay_tolerance();

        if arrival >= allowed_at {
            if arrival <= self.tat {
                self.burst = self.burst.saturating_sub(1);
            } else {
                self.burst = (self.burst + 1).min(self.max_burst);
            }

            self.tat = tat + self.emission_interval();
            Ok(())
        } else {
            Err(allowed_at)
        }
    }
}

pub(crate) struct RateLimitedRequest {
    subdomain: Subdomain,
    method: MethodId,
    inner: RequestBuilder,
}

impl RateLimitedRequest {
    pub(crate) fn new(subdomain: Subdomain, method: MethodId, inner: RequestBuilder) -> Self {
        Self {
            subdomain,
            method,
            inner,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_rate_limit() {
        let mut bucket =
            RateLimitBucket::with_arrival(Instant::now(), Duration::from_secs(1), 10, 5);

        let mut now = Instant::now();

        for _ in 0..10 {
            now += bucket.emission_interval();
            bucket.check_arrival(now).unwrap();
        }

        assert_eq!(
            bucket.check_arrival(now).unwrap_err(),
            now + bucket.emission_interval()
        );

        now += bucket.emission_interval();
        bucket.check_arrival(now).unwrap();
    }
}
