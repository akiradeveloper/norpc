use tokio::sync::oneshot;

/// Send support.
/// `#[service]` generates Send futures.
pub mod send;

/// non-Send support.
/// `#[service(?Send)]` generates non-Send futures.
pub mod no_send;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to send a request")]
    SendError,
    #[error("failed to receive a response")]
    RecvError,
}

pub struct Request<X, Y> {
    inner: X,
    tx: oneshot::Sender<Y>,
}
