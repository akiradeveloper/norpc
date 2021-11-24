use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tower_service::Service;

// Re-exported for compiler
pub use async_trait::async_trait;
pub use futures::future::poll_fn;

pub use norpc_macros::service;

#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    #[error("failed to send a request")]
    SendError,
    #[error("failed to receive response")]
    RecvError,
}

#[derive(Debug)]
pub enum Error<AppError> {
    AppError(AppError),
    TransportError(TransportError),
}

/// mpsc channel wrapper on the client-side.
pub struct ClientChannel<X, Y, AppError> {
    tx: mpsc::UnboundedSender<Request<X, Result<Y, AppError>>>,
}
impl<X, Y, AppError> ClientChannel<X, Y, AppError> {
    pub fn new(tx: mpsc::UnboundedSender<Request<X, Result<Y, AppError>>>) -> Self {
        Self { tx }
    }
}
impl<X, Y, AppError> Clone for ClientChannel<X, Y, AppError> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
impl<X: 'static + Send, Y: 'static + Send, AppError: 'static + Send> Service<X>
    for ClientChannel<X, Y, AppError>
{
    type Response = Y;
    type Error = Error<AppError>;
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
        Box::pin(async move {
            let (tx1, rx1) = oneshot::channel::<Result<Y, AppError>>();
            let req = Request {
                inner: req,
                tx: tx1,
            };
            tx.send(req)
                .map_err(|_| Error::TransportError(TransportError::SendError))?;
            let rep = rx1
                .await
                .map_err(|_| Error::TransportError(TransportError::RecvError))?;
            rep.map_err(|e| Error::AppError(e))
        })
    }
}

/// Pair of user defined request and
/// a oneshot sender for the response.
pub struct Request<X, Y> {
    inner: X,
    tx: oneshot::Sender<Y>,
}

/// mpsc channel wrapper on the server-side.
pub struct ServerChannel<Req, Svc: Service<Req>> {
    service: Svc,
    rx: mpsc::UnboundedReceiver<Request<Req, Result<Svc::Response, Svc::Error>>>,
}
impl<Req, Svc: Service<Req> + 'static + Send> ServerChannel<Req, Svc>
where
    Req: 'static + Send,
    Svc::Future: Send,
    Svc::Response: Send,
    Svc::Error: Send,
{
    pub fn new(
        rx: mpsc::UnboundedReceiver<Request<Req, Result<Svc::Response, Svc::Error>>>,
        service: Svc,
    ) -> Self {
        Self { service, rx }
    }
    pub async fn serve(mut self) {
        while let Some(Request { tx, inner }) = self.rx.recv().await {
            // back-pressure
            futures::future::poll_fn(|ctx| self.service.poll_ready(ctx))
                .await
                .ok();
            let fut = self.service.call(inner);
            tokio::spawn(async {
                let rep = fut.await;
                tx.send(rep).ok();
            });
        }
    }
}
