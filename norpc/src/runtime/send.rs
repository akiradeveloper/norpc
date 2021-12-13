use tokio::sync::mpsc;
use tokio::sync::oneshot;

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
impl<X: 'static + Send, Y: 'static + Send> crate::Service<X> for ClientService<X, Y> {
    type Response = Y;
    type Error = Error;
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

pub struct ServerExecutor<X, Svc: crate::Service<X>> {
    service: Svc,
    rx: mpsc::Receiver<Request<X, Svc::Response>>,
}
impl<X, Svc: crate::Service<X> + 'static + Send> ServerExecutor<X, Svc>
where
    X: 'static + Send,
    Svc::Future: Send,
    Svc::Response: Send,
{
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
            tokio::spawn(async move {
                if let Ok(rep) = fut.await {
                    tx.send(rep).ok();
                }
            });
        }
    }
}
