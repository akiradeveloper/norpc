#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(hidden)]
pub use async_trait::async_trait;
#[doc(hidden)]
pub use futures::future::poll_fn;
#[doc(hidden)]
pub use tower_service::Service;

/// Macro for code-generation.
pub use norpc_macros::service;

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
/// The default runtime implementation using Tokio.
pub mod runtime;
