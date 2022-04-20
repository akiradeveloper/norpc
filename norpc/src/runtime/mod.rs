#[cfg(feature = "runtime-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime-tokio")))]
/// Tokio support.
/// `#[service]` generates Send futures.
pub mod tokio;