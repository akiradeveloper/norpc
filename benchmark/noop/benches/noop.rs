use tokio::sync::mpsc;
use tower::Service;

#[norpc::service]
trait Noop {
    fn noop();
}

#[derive(Clone)]
struct NoopApp;
#[norpc::async_trait]
impl Noop for NoopApp {
    async fn noop(self) {
        // tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_noop(c: &mut Criterion) {
    use norpc::runtime::*;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let (tx, rx) = mpsc::channel(100);
    rt.spawn(async move {
        let app = NoopApp;
        let service = NoopService::new(app);
        let server = send::ServerExecutor::new(rx, service);
        server.serve().await
    });
    let chan = send::ClientService::new(tx);
    let cli = NoopClient::new(chan);
    c.bench_with_input(BenchmarkId::new("noop request", 1), &cli, |b, cli| {
        b.to_async(&rt).iter(|| async {
            let mut cli = cli.clone();
            cli.noop().await.unwrap();
        });
    });
}

fn bench_channel(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let (tx, mut rx) = mpsc::channel(100);
    rt.spawn(async move { while let Some(()) = rx.recv().await {} });
    c.bench_function("noop channel", |b| {
        b.to_async(&rt).iter(|| async {
            tx.send(()).await.unwrap();
        })
    });
}

criterion_group!(noop, bench_noop, bench_channel);
criterion_main!(noop);
