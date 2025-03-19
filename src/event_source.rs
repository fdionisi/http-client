use anyhow::{anyhow, Result};
use futures::AsyncBufReadExt;
use futures::{io::BufReader, Stream};
use tracing::{debug, error, info, warn};

use crate::{AsyncBody, HttpClient, Request};

pub enum EventSourceFragment {
    Comment(String),
    Data(String),
    Event(String),
    Id(String),
    Retry(String),
}

pub trait EventSource {
    fn event_source_fragments(
        &self,
        request: Request<AsyncBody>,
    ) -> impl Stream<Item = Result<EventSourceFragment>> + Send;
}

impl<T> EventSource for T
where
    T: HttpClient + Send + 'static,
{
    fn event_source_fragments(
        &self,
        mut request: Request<AsyncBody>,
    ) -> impl Stream<Item = Result<EventSourceFragment>> + Send {
        let client = self;
        debug!("Creating event source fragments stream");

        async_stream::stream! {
            let headers = request.headers_mut();
            headers.append("Accept", "text/event-stream".parse()?);

            match client.send(request).await {
                Ok(mut response) => {
                    info!("Received response from event source");

                    if response.status() != 200 {
                        error!("Event source returned non-200 status: {}", response.status());
                        yield Err(anyhow!("Event source returned non-200 status: {}", response.status()));
                        return;
                    }

                    let body = response.body_mut();
                    let mut reader = BufReader::new(body);

                    loop {
                        let mut line = String::new();
                        let read_result = reader.read_line(&mut line).await;

                        match read_result {
                            Ok(0) => {
                                debug!("Connection ended");
                                break;
                            },
                            Ok(_) => {
                                line = line.trim_end().to_string();
                                debug!("Received line: {}", line);

                                if line.starts_with(":") {
                                    let value = line.trim_start_matches(":").trim_start().to_string();
                                    debug!("Parsed comment: {}", value);
                                    yield Ok(EventSourceFragment::Comment(value));
                                } else if line.starts_with("data:") {
                                    let value = line.trim_start_matches("data:").trim_start().to_string();
                                    debug!("Parsed data: {}", value);
                                    yield Ok(EventSourceFragment::Data(value));
                                } else if line.starts_with("event:") {
                                    let value = line.trim_start_matches("event:").trim_start().to_string();
                                    debug!("Parsed event: {}", value);
                                    yield Ok(EventSourceFragment::Event(value));
                                } else if line.starts_with("id:") {
                                    let value = line.trim_start_matches("id:").trim_start().to_string();
                                    debug!("Parsed id: {}", value);
                                    yield Ok(EventSourceFragment::Id(value));
                                } else if line.starts_with("retry:") {
                                    let value = line.trim_start_matches("retry:").trim_start().to_string();
                                    debug!("Parsed retry: {}", value);
                                    yield Ok(EventSourceFragment::Retry(value));
                                } else {
                                    warn!("Received unrecognized event source line: {}", line);
                                }
                            },
                            Err(e) => {
                                error!("Error reading line from event source: {:?}", e);
                                break;
                            },
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to send request to event source: {:?}", e);
                    yield Err(anyhow!(e));
                }
            }
        }
    }
}
