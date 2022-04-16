use std::marker::PhantomData;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

struct Request<X, Y> {
    inner: X,
    tx: oneshot::Sender<Y>,
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
        let (tx, rx) = mpsc::channel(100);
        let server = Server::new(rx, self.svc);
        let chan = Channel::new(tx);
        (chan, server)
    }
}

pub struct Channel<X, Y> {
    tx: mpsc::Sender<Request<X, Y>>,
}
impl<X, Y> Channel<X, Y> {
    fn new(tx: mpsc::Sender<Request<X, Y>>) -> Self {
        Self { tx: tx }
    }
}
impl<X, Y> Clone for Channel<X, Y> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
impl<X: 'static + Send, Y: 'static + Send> crate::Service<X> for Channel<X, Y> {
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
        Box::pin(async move {
            let (tx1, rx1) = oneshot::channel::<Y>();
            let req = Request {
                inner: req,
                tx: tx1,
            };
            if tx.send(req).await.is_err() {
                anyhow::bail!("failed to send a request");
            }
            let rep = rx1.await?;
            Ok(rep)
        })
    }
}

pub struct Server<X, Svc: crate::Service<X>> {
    service: Svc,
    rx: mpsc::Receiver<Request<X, Svc::Response>>,
}
impl<X, Svc: crate::Service<X> + 'static + Send> Server<X, Svc>
where
    X: 'static + Send,
    Svc::Future: Send,
    Svc::Response: Send,
{
    fn new(rx: mpsc::Receiver<Request<X, Svc::Response>>, service: Svc) -> Self {
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
