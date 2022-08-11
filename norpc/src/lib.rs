#![cfg_attr(docsrs, feature(doc_cfg))]

//! norpc is a library to implement in-process microservices.
//!
//! ```
//! #[norpc::service]
//! trait HelloWorld {
//!    fn hello(s: String) -> String;
//! }
//! struct HelloWorldApp;
//! #[async_trait::async_trait]
//! impl HelloWorld for HelloWorldApp {
//!    async fn hello(&self, s: String) -> String {
//!        format!("Hello, {}", s)
//!    }
//! }
//! let rep = tokio_test::block_on(async {
//!     use norpc::runtime::*;
//!     let app = HelloWorldApp;
//!     let svc = HelloWorldService::new(app);
//!     let (chan, server) = ServerBuilder::new(svc).build();
//!     ::tokio::spawn(server.serve(tokio::TokioExecutor));
//!     let mut cli = HelloWorldClient::new(chan);
//!     cli.hello("World".to_owned()).await
//! });
//! assert_eq!(rep, "Hello, World");
//! ```

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
/// Runtime implementation.
pub mod runtime;
