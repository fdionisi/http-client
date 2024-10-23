use std::{
    borrow::Cow,
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{AsyncRead, Stream};
use pin_project_lite::pin_project;

pin_project! {
    /// Based on the implementation of AsyncBody in
    /// https://github.com/zed-industries/zed/blob/d53a86b01dd3d02980938cbce1bfd74e35901dda/crates/http_client/src/async_body.rs
    #[derive(Clone)]
    pub struct AsyncBody{
        #[pin]
        inner: Inner
    }
}

pub enum Inner {
    /// An empty body.
    Empty,

    /// A body stored in memory.
    SyncReader(std::io::Cursor<Cow<'static, [u8]>>),

    /// An asynchronous reader.
    AsyncReader(Pin<Box<dyn futures::AsyncRead + Send + Sync>>),
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        match self {
            Inner::Empty => Inner::Empty,
            Inner::SyncReader(reader) => Inner::SyncReader(reader.clone()),
            Inner::AsyncReader(_) => panic!("Cannot clone an async reader"),
        }
    }
}

impl AsyncBody {
    /// Create a new empty body.
    ///
    /// An empty body represents the *absence* of a body, which is semantically
    /// different than the presence of a body of zero length.
    pub fn empty() -> Self {
        Self {
            inner: Inner::Empty,
        }
    }

    /// Create a streaming body that reads from the given reader.
    pub fn from_reader<R>(read: R) -> Self
    where
        R: AsyncRead + Send + Sync + 'static,
    {
        Self {
            inner: Inner::AsyncReader(Box::pin(read)),
        }
    }
}

impl Default for AsyncBody {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<()> for AsyncBody {
    fn from(_: ()) -> Self {
        Self::empty()
    }
}

impl From<Vec<u8>> for AsyncBody {
    fn from(body: Vec<u8>) -> Self {
        Self {
            inner: Inner::SyncReader(std::io::Cursor::new(Cow::Owned(body))),
        }
    }
}

impl From<&'_ [u8]> for AsyncBody {
    fn from(body: &[u8]) -> Self {
        body.to_vec().into()
    }
}

impl From<String> for AsyncBody {
    fn from(body: String) -> Self {
        body.into_bytes().into()
    }
}

impl From<&'_ str> for AsyncBody {
    fn from(body: &str) -> Self {
        body.as_bytes().into()
    }
}

impl<T: Into<Self>> From<Option<T>> for AsyncBody {
    fn from(body: Option<T>) -> Self {
        match body {
            Some(body) => body.into(),
            None => Self::empty(),
        }
    }
}

impl futures::AsyncRead for Inner {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        // SAFETY: Standard Enum pin projection
        let inner = unsafe { &mut self.get_unchecked_mut() };
        match inner {
            Inner::Empty => Poll::Ready(Ok(0)),
            // Blocking call is over an in-memory buffer
            Inner::SyncReader(cursor) => Poll::Ready(cursor.read(buf)),
            Inner::AsyncReader(async_reader) => {
                AsyncRead::poll_read(async_reader.as_mut(), cx, buf)
            }
        }
    }
}

impl futures::AsyncRead for AsyncBody {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut this = self.project();
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}

impl Stream for AsyncBody {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let mut buffer = vec![0; 1024];

        match this.inner.as_mut().poll_read(cx, &mut buffer) {
            Poll::Ready(Ok(0)) => Poll::Ready(None),
            Poll::Ready(Ok(n)) => {
                buffer.truncate(n);
                Poll::Ready(Some(Ok(buffer)))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Read for AsyncBody {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        futures::executor::block_on(futures::AsyncReadExt::read(self, buf))
    }
}
