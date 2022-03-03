use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tower_service::Service;

use super::{Error, Request};

pub struct ClientService<X, Y> {
    tx: mpsc::Sender<Request<X, Y>>,
}
impl<X, Y> ClientService<X, Y> {
    pub fn new(tx: mpsc::Sender<Request<X, Y>>) -> Self {
        Self { tx: tx }
    }
}
impl<X, Y> Clone for ClientService<X, Y> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
impl<X: 'static, Y: 'static> Service<X> for ClientService<X, Y> {
    type Response = Y;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Y, Self::Error>>>>;

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
            tx.send(req).await.map_err(|_| Error::SendError)?;
            let rep = rx1.await.map_err(|_| Error::RecvError)?;
            Ok(rep)
        })
    }
}

pub struct ServerExecutor<X, Svc: Service<X>> {
    service: Svc,
    rx: mpsc::Receiver<Request<X, Svc::Response>>,
}
impl<X: 'static, Svc: Service<X> + 'static> ServerExecutor<X, Svc> {
    pub fn new(rx: mpsc::Receiver<Request<X, Svc::Response>>, service: Svc) -> Self {
        Self { service, rx: rx }
    }
    pub async fn serve(mut self) {
        while let Some(Request { tx, inner }) = self.rx.recv().await {
            // backpressure
            crate::poll_fn(|ctx| self.service.poll_ready(ctx))
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