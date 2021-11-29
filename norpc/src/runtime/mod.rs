use tokio::sync::oneshot;

/// Send support.
/// `#[service]` generates Send futures.
pub mod send;

/// non-Send support.
/// `#[service(?Send)]` generates non-Send futures.
pub mod no_send;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to send a request (closed)")]
    SendClosed,
    #[error("failed to receive response (closed)")]
    RecvClosed,
}

pub struct Request<X, Y> {
    inner: X,
    tx: oneshot::Sender<Y>,
}
