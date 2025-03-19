use std::{future::Future, pin::Pin};

use anyhow::Result;
use futures::{stream::StreamExt, TryStreamExt};
use http_client::{
    http::{Request, Response},
    AsyncBody, HttpClient,
};
use reqwest::Client;
use tracing::{debug, error, info, warn};

pub struct HttpClientReqwest(Client);

impl Default for HttpClientReqwest {
    fn default() -> Self {
        debug!("Creating default HttpClientReqwest");
        Self(Client::new())
    }
}

impl From<Client> for HttpClientReqwest {
    fn from(client: Client) -> Self {
        debug!("Creating HttpClientReqwest from Client");
        Self(client)
    }
}

impl HttpClient for HttpClientReqwest {
    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<AsyncBody>>> + Send + '_>> {
        debug!("Sending request to {}", request.uri());
        let reqwest_request = match self
            .0
            .request(request.method().clone(), request.uri().to_string())
            .headers(request.headers().clone())
            .body(reqwest::Body::wrap_stream(request.into_body()))
            .build()
        {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to build reqwest request: {}", e);
                panic!("Failed to build reqwest request: {}", e);
            }
        };

        let client = self.0.clone();
        Box::pin(async move {
            let response = match client.execute(reqwest_request).await {
                Ok(resp) => {
                    info!("Request succeeded with status: {}", resp.status());
                    resp
                }
                Err(e) => {
                    error!("Request failed: {}", e);
                    panic!("Request execution failed: {}", e);
                }
            };

            let mut res = Response::builder().status(response.status());
            for (key, value) in response.headers().iter() {
                res = res.header(key, value);
                debug!("Response header: {} = {:?}", key, value);
            }
            debug!("Creating streaming body for response");

            let reader = response
                .bytes_stream()
                .map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
                .into_async_read();

            let async_body = AsyncBody::from_reader(Box::pin(reader));

            match res.body(async_body) {
                Ok(r) => {
                    debug!("Response body processed successfully");
                    Ok(r)
                }
                Err(e) => {
                    warn!("Failed to build response: {}", e);
                    panic!("Failed to build response: {}", e);
                }
            }
        })
    }
}
