use tokio::sync::oneshot;

/// Send support.
/// `#[service]` generates Send futures.
pub mod send;

/// non-Send support.
/// `#[service(?Send)]` generates non-Send futures.
// pub mod no_send;

pub struct Request<X, Y> {
    inner: X,
    tx: oneshot::Sender<anyhow::Result<Y>>,
}
