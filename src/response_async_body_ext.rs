use std::{future::Future, pin::Pin};

use anyhow::anyhow;
use futures::{Stream, StreamExt, TryStreamExt};
use http::Response;
use serde::de::DeserializeOwned;

use crate::AsyncBody;

pub trait ResponseAsyncBodyExt {
    fn stream(self) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, anyhow::Error>> + Send + Sync>>;

    fn stream_json<D>(self) -> Pin<Box<dyn Stream<Item = Result<D, anyhow::Error>> + Send>>
    where
        D: DeserializeOwned;

    fn bytes(self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, http::Error>> + Send + Sync>>;

    fn json<D>(self) -> Pin<Box<dyn Future<Output = Result<D, http::Error>> + Send + Sync>>
    where
        D: DeserializeOwned;

    fn text(self) -> Pin<Box<dyn Future<Output = Result<String, http::Error>> + Send + Sync>>;
}

impl ResponseAsyncBodyExt for Response<AsyncBody> {
    fn stream(self) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, anyhow::Error>> + Send + Sync>> {
        Box::pin(self.into_body().map_err(|err| anyhow!(err)))
    }

    fn stream_json<D>(self) -> Pin<Box<dyn Stream<Item = Result<D, anyhow::Error>> + Send>>
    where
        D: DeserializeOwned,
    {
        self.stream()
            .map_ok(|bytes| serde_json::from_slice::<D>(&bytes).unwrap())
            .map_err(anyhow::Error::from)
            .boxed()
    }

    fn bytes(self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, http::Error>> + Send + Sync>> {
        Box::pin(async move {
            let body: Vec<Vec<u8>> = self.into_body().try_collect().await.unwrap();
            Ok(body.into_iter().flatten().collect())
        })
    }

    fn json<D>(self) -> Pin<Box<dyn Future<Output = Result<D, http::Error>> + Send + Sync>>
    where
        D: DeserializeOwned,
    {
        Box::pin(async move {
            let bytes = self.bytes().await?;
            Ok(serde_json::from_slice(bytes.as_slice()).unwrap())
        })
    }

    fn text(self) -> Pin<Box<dyn Future<Output = Result<String, http::Error>> + Send + Sync>> {
        Box::pin(async move {
            let bytes = self.bytes().await?;
            Ok(String::from_utf8(bytes).unwrap())
        })
    }
}
