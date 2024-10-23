use std::{future::Future, pin::Pin};

use http_client::{
    http::{Error, Request, Response},
    AsyncBody, HttpClient,
};
use reqwest::Client;

pub struct HttpClientReqwest(Client);

impl Default for HttpClientReqwest {
    fn default() -> Self {
        Self(Client::new())
    }
}

impl From<Client> for HttpClientReqwest {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

impl HttpClient for HttpClientReqwest {
    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<AsyncBody>, Error>> + Send + Sync>> {
        let reqwest_request = self
            .0
            .request(request.method().clone(), request.uri().to_string())
            .headers(request.headers().clone())
            .body(reqwest::Body::wrap_stream(request.into_body()))
            .build()
            .unwrap();

        let client = self.0.clone();
        Box::pin(async move {
            let response = client.execute(reqwest_request).await.unwrap();

            let mut res = Response::builder().status(response.status());
            for (key, value) in response.headers().iter() {
                res = res.header(key, value);
            }

            Ok(res
                .body(response.bytes().await.unwrap().to_vec().into())
                .unwrap())
        })
    }
}
