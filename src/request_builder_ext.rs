use http::{
    request::{Builder as RequestBuilder, Request},
    HeaderMap,
};
use serde::Serialize;

use crate::AsyncBody;

pub trait RequestBuilderExt {
    fn headers(self, headers: HeaderMap) -> Self;

    fn end(self) -> Result<Request<AsyncBody>, http::Error>;

    fn json<S>(self, payload: S) -> Result<Request<AsyncBody>, http::Error>
    where
        S: Serialize;
}

impl RequestBuilderExt for RequestBuilder {
    fn end(self) -> Result<Request<AsyncBody>, http::Error> {
        self.body(AsyncBody::empty())
    }

    fn json<S>(self, payload: S) -> Result<Request<AsyncBody>, http::Error>
    where
        S: Serialize,
    {
        self.body(serde_json::to_vec(&payload).unwrap().into())
    }

    fn headers(mut self, headers: HeaderMap) -> Self {
        let h = self.headers_mut().unwrap();
        for (key, value) in headers.iter() {
            h.insert(key, value.clone());
        }
        self
    }
}
