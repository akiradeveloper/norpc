use std::marker::PhantomData;

use futures::channel::{mpsc, oneshot};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower::util::BoxCloneService;
use tower::Layer;
use tower::Service;

enum CoreRequest<X, Y> {
    AppRequest {
        inner: X,
        tx: oneshot::Sender<Y>,
        stream_id: u64,
    },
    Cancel {
        stream_id: u64,
    },
}

pub struct ServerBuilder<X, Svc> {
    svc: Svc,
    phantom_x: PhantomData<X>,
}
impl<X, Svc: crate::Service<X> + 'static + Send> ServerBuilder<X, Svc>
where
    X: 'static + Send,
    Svc::Future: Send,
    Svc::Response: Send,
{
    pub fn new(svc: Svc) -> Self {
        Self {
            svc: svc,
            phantom_x: PhantomData,
        }
    }
    pub fn build(self) -> (Channel<X, Svc::Response>, Server<X, Svc>) {
        let (tx, rx) = mpsc::unbounded();
        let server = Server::new(rx, self.svc);
        let inner = InnerChannel::new(tx);
        let chan = Channel {
            inner: BoxCloneService::new(inner),
        };
        (chan, server)
    }
}

pub struct Channel<X, Y> {
    inner: BoxCloneService<X, Y, anyhow::Error>,
}
impl<X, Y> Channel<X, Y> {
    pub fn new<S: Service<X, Response = Y, Error = anyhow::Error> + 'static + Send + Clone>(
        inner: S,
    ) -> Self
    where
        S::Future: 'static + Send,
    {
        Self {
            inner: BoxCloneService::new(inner),
        }
    }
    pub fn unwrap(self) -> impl Service<X> {
        self.inner
    }
}

unsafe impl<X, Y> Sync for Channel<X, Y> {}
impl<X, Y> Clone for Channel<X, Y> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<X: 'static + Send, Y: 'static + Send> Service<X> for Channel<X, Y> {
    type Response = Y;
    type Error = anyhow::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Y, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: X) -> Self::Future {
        self.inner.call(req)
    }
}

pub struct InnerChannel<X, Y> {
    next_id: Arc<AtomicU64>,
    stream_id: u64,
    tx: mpsc::UnboundedSender<CoreRequest<X, Y>>,
}
impl<X, Y> InnerChannel<X, Y> {
    fn new(tx: mpsc::UnboundedSender<CoreRequest<X, Y>>) -> Self {
        Self {
            stream_id: 0,
            next_id: Arc::new(AtomicU64::new(1)),
            tx: tx,
        }
    }
}
impl<X, Y> Clone for InnerChannel<X, Y> {
    fn clone(&self) -> Self {
        let next_id = self.next_id.clone();
        let stream_id = next_id.fetch_add(1, Ordering::SeqCst);
        Self {
            stream_id,
            next_id: next_id,
            tx: self.tx.clone(),
        }
    }
}
impl<X, Y> Drop for InnerChannel<X, Y> {
    fn drop(&mut self) {
        let cancel_req = CoreRequest::Cancel {
            stream_id: self.stream_id,
        };
        self.tx.unbounded_send(cancel_req).ok();
    }
}
impl<X: 'static + Send, Y: 'static + Send> crate::Service<X> for InnerChannel<X, Y> {
    type Response = Y;
    type Error = anyhow::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Y, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: X) -> Self::Future {
        let tx = self.tx.clone();
        let stream_id = self.stream_id;
        Box::pin(async move {
            let (tx1, rx1) = oneshot::channel::<Y>();
            let req = CoreRequest::AppRequest {
                inner: req,
                tx: tx1,
                stream_id,
            };
            if tx.unbounded_send(req).is_err() {
                anyhow::bail!("failed to send a request");
            }
            let rep = rx1.await?;
            Ok(rep)
        })
    }
}

pub struct Server<X, Svc: crate::Service<X>> {
    service: Svc,
    rx: mpsc::UnboundedReceiver<CoreRequest<X, Svc::Response>>,
}
impl<X, Svc: crate::Service<X> + 'static + Send> Server<X, Svc>
where
    X: 'static + Send,
    Svc::Future: Send,
    Svc::Response: Send,
{
    fn new(rx: mpsc::UnboundedReceiver<CoreRequest<X, Svc::Response>>, service: Svc) -> Self {
        Self { service, rx: rx }
    }
    pub async fn serve(mut self, executor: impl futures::task::Spawn) {
        use futures::future::AbortHandle;
        use futures::task::SpawnExt;
        let mut processings: HashMap<u64, AbortHandle> = HashMap::new();
        while let Some(req) = self.rx.next().await {
            match req {
                CoreRequest::AppRequest {
                    inner,
                    tx,
                    stream_id,
                } => {
                    if let Some(handle) = processings.get(&stream_id) {
                        handle.abort();
                    }
                    processings.remove(&stream_id);

                    // backpressure
                    crate::poll_fn(|ctx| self.service.poll_ready(ctx))
                        .await
                        .ok();
                    let fut = self.service.call(inner);
                    let (fut, abort_handle) = futures::future::abortable(async move {
                        if let Ok(rep) = fut.await {
                            tx.send(rep).ok();
                        }
                    });
                    let fut = async move {
                        fut.await.ok();
                    };
                    if let Err(e) = executor.spawn(fut) {
                        abort_handle.abort();
                    }
                    processings.insert(stream_id, abort_handle);
                }
                CoreRequest::Cancel { stream_id } => {
                    if let Some(handle) = processings.get(&stream_id) {
                        handle.abort();
                    }
                    processings.remove(&stream_id);
                }
            }
        }
    }
}

#[cfg(feature = "tokio-executor")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-executor")))]
/// Tokio support.
pub struct TokioExecutor;

#[cfg(feature = "tokio-executor")]
impl futures::task::Spawn for TokioExecutor {
    fn spawn_obj(
        &self,
        future: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        tokio::spawn(future);
        Ok(())
    }
}

#[cfg(feature = "async-std-executor")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std-executor")))]
/// async-std support.
pub struct AsyncStdExecutor;

#[cfg(feature = "async-std-executor")]
impl futures::task::Spawn for AsyncStdExecutor {
    fn spawn_obj(
        &self,
        future: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        async_std::task::spawn(future);
        Ok(())
    }
}
