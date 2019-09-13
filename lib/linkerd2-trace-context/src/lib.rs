use bytes::Bytes;
use futures::Sink;
use linkerd2_error::Error;
use rand::Rng;
use std::fmt;
use std::time::SystemTime;

pub mod layer;
mod propagation;

pub use layer::layer;

const SPAN_ID_LEN: usize = 8;

#[derive(Debug, Default)]
pub struct Id(Vec<u8>);

#[derive(Debug, Default)]
pub struct Flags(u8);

#[derive(Debug)]
pub struct Span {
    pub trace_id: Id,
    pub span_id: Id,
    pub parent_id: Id,
    pub span_name: String,
    pub start: SystemTime,
    pub end: SystemTime,
}

pub trait SpanSink {
    fn try_send(&mut self, span: Span) -> Result<(), Error>;
}

impl<S> SpanSink for S
where
    S: Sink<SinkItem = Span>,
    S::SinkError: Into<Error>,
{
    fn try_send(&mut self, span: Span) -> Result<(), Error> {
        self.start_send(span).map(|_| ()).map_err(Into::into)
    }
}

// === impl Id ===

impl Id {
    fn new_span_id<R: Rng>(rng: &mut R) -> Self {
        let mut bytes = vec![0; SPAN_ID_LEN];
        rng.fill(bytes.as_mut_slice());
        Self(bytes)
    }
}

impl Into<Vec<u8>> for Id {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<Vec<u8>> for Id {
    fn as_ref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0.iter() {
            write!(f, "{:02x?}", b)?;
        }
        Ok(())
    }
}

impl From<Bytes> for Id {
    fn from(buf: Bytes) -> Self {
        Id(buf.to_vec())
    }
}

// === impl Flags ===

impl Flags {
    pub fn is_sampled(&self) -> bool {
        self.0 & 1 == 1
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x?}", self.0)
    }
}

impl From<Bytes> for Flags {
    fn from(buf: Bytes) -> Self {
        Flags(buf[0])
    }
}
