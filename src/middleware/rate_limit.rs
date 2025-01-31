use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorTooManyRequests;
use actix_web::Error;
use futures::future::{ok, Ready};
use once_cell::sync::Lazy;

static RATE_LIMITER: Lazy<Mutex<HashMap<String, Vec<u64>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct RateLimiter {
    pub requests_per_minute: u32,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self { requests_per_minute }
    }
}

impl<S> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service,
            requests_per_minute: self.requests_per_minute,
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: S,
    requests_per_minute: u32,
}

impl<S> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut rate_limiter = RATE_LIMITER.lock().unwrap();
        let requests = rate_limiter.entry(ip.clone()).or_insert_with(Vec::new());

        // Remove requests older than 1 minute
        requests.retain(|&timestamp| now - timestamp < 60);

        if requests.len() >= self.requests_per_minute as usize {
            return Box::pin(futures::future::err(ErrorTooManyRequests(
                "Rate limit exceeded. Please try again later.",
            )));
        }

        requests.push(now);
        drop(rate_limiter);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

pub fn cleanup_rate_limiter() {
    let mut rate_limiter = RATE_LIMITER.lock().unwrap();
    rate_limiter.clear();
}