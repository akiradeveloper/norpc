use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tower_service::Service;

pub mod build;
mod compiler;

pub use async_trait::async_trait;

#[macro_export]
macro_rules! include_code {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}

#[derive(Debug)]
pub enum Error<AppError> {
    AppError(AppError),
    ChanError(anyhow::Error),
}

pub struct Channel<X, Y> {
    tx: mpsc::Sender<Request<X, Y>>,
}
impl<X, Y> Channel<X, Y> {
    pub fn new(tx: mpsc::Sender<Request<X, Y>>) -> Self {
        Self { tx }
    }
    pub async fn call(self, req: X) -> anyhow::Result<Y> {
        let (tx1, rx1) = oneshot::channel::<Y>();
        let req = Request {
            inner: req,
            tx: tx1,
        };
        self.tx.try_send(req).ok();
        let rep = rx1.await.unwrap();
        Ok(rep)
    }
}
impl<X, Y> Clone for Channel<X, Y> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub struct Request<X, Y> {
    inner: X,
    tx: oneshot::Sender<Y>,
}
pub struct Server<Req, Svc: Service<Req>> {
    service: Svc,
    rx: mpsc::Receiver<Request<Req, Svc::Response>>,
}
impl<Req, Svc: Service<Req> + 'static + Send + Clone> Server<Req, Svc>
where
    Req: 'static + Send,
    Svc::Future: Send,
    Svc::Response: Send,
{
    pub fn new(rx: mpsc::Receiver<Request<Req, Svc::Response>>, service: Svc) -> Self {
        Self { service, rx }
    }
    pub async fn serve(mut self) {
        while let Some(Request { tx, inner }) = self.rx.recv().await {
            let mut cln = self.service.clone();
            tokio::spawn(async move {
                if let Ok(rep) = cln.call(inner).await {
                    tx.send(rep);
                }
            });
        }
    }
}
