mod async_body;
mod event_source;
mod request_builder_ext;
mod response_async_body_ext;

use std::{future::Future, pin::Pin};

pub use anyhow::Result;
pub use http::{self, Error, Request, Response};

pub use crate::{
    async_body::AsyncBody,
    event_source::{EventSource, EventSourceFragment},
    request_builder_ext::RequestBuilderExt,
    response_async_body_ext::ResponseAsyncBodyExt,
};

pub trait HttpClient: Send + Sync {
    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<AsyncBody>>> + Send + '_>>;
}

impl<R> HttpClient for R
where
    R: AsRef<dyn HttpClient> + Send + Sync,
{
    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<AsyncBody>>> + Send + '_>> {
        self.as_ref().send(request)
    }
}
