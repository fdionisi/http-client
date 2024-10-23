mod async_body;
mod request_builder_ext;
mod response_async_body_ext;

use std::{future::Future, pin::Pin};

// use anyhow::Result;
pub use http;
pub use http::{Error, Request, Response};

pub use crate::{
    async_body::AsyncBody, request_builder_ext::RequestBuilderExt,
    response_async_body_ext::ResponseAsyncBodyExt,
};

/// A trait for defining an abstract HTTP client, runtime-agnostic.
pub trait HttpClient: Send + Sync {
    /// Send an HTTP request and return the response.
    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<AsyncBody>, Error>> + Send + Sync>>;
}

impl<R> HttpClient for R
where
    R: AsRef<dyn HttpClient> + Send + Sync,
{
    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<AsyncBody>, Error>> + Send + Sync>> {
        self.as_ref().send(request)
    }
}
