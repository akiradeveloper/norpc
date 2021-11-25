use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tower_service::Service;

use crate::{Request, Error};

/// mpsc channel wrapper on the client-side.
pub struct ClientChannel<X, Y> {
    tx: mpsc::UnboundedSender<Request<X, Y>>,
}
impl<X, Y> ClientChannel<X, Y> {
    pub fn new(tx: mpsc::UnboundedSender<Request<X, Y>>) -> Self {
        Self { tx }
    }
}
impl<X, Y> Clone for ClientChannel<X, Y> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
impl<X: 'static, Y: 'static> Service<X> for ClientChannel<X, Y> {
    type Response = Y;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Y, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: X) -> Self::Future {
        let tx = self.tx.clone();
        Box::pin(async move {
            let (tx1, rx1) = oneshot::channel::<Y>();
            let req = Request {
                inner: req,
                tx: tx1,
            };
            tx.send(req).map_err(|_| Error::SendError)?;
            let rep = rx1.await.map_err(|_| Error::RecvError)?;
            Ok(rep)
        })
    }
}

/// mpsc channel wrapper on the server-side.
pub struct ServerChannel<Req, Svc: Service<Req>> {
    service: Svc,
    rx: mpsc::UnboundedReceiver<Request<Req, Svc::Response>>,
}
impl<Req: 'static, Svc: Service<Req> + 'static> ServerChannel<Req, Svc> {
    pub fn new(rx: mpsc::UnboundedReceiver<Request<Req, Svc::Response>>, service: Svc) -> Self {
        Self { service, rx }
    }
    pub async fn serve(mut self) {
        while let Some(Request { tx, inner }) = self.rx.recv().await {
            // back-pressure
            futures::future::poll_fn(|ctx| self.service.poll_ready(ctx))
                .await
                .ok();
            let fut = self.service.call(inner);
            tokio::task::spawn_local(async move {
                if let Ok(rep) = fut.await {
                    tx.send(rep).ok();
                }
            });
        }
    }
}
