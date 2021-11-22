use tarpc::{client, context, server};

#[tarpc::service]
trait Noop {
    async fn noop() -> ();
}
#[derive(Clone)]
struct NoopApp;
#[tarpc::server]
impl Noop for NoopApp {
    async fn noop(self, _: context::Context) -> () {}
}

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_noop_tarpc(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    // This enables tokio::xxx access the default runtime.
    let _guard = rt.enter();
    let (tx, rx) = tarpc::transport::channel::unbounded();
    tokio::spawn({
        use tarpc::server::Channel;
        let server = server::BaseChannel::new(server::Config::default(), rx);
        let service = NoopApp;
        server.requests().execute(service.serve())
    });
    let cli = NoopClient::new(client::Config::default(), tx).spawn();
    c.bench_with_input(
        BenchmarkId::new("noop request (tarpc)", 1),
        &cli,
        |b, cli| {
            let ctx = context::current();
            b.to_async(&rt).iter(|| async {
                cli.noop(ctx).await.unwrap();
            });
        },
    );
}

criterion_group!(noop_tarpc, bench_noop_tarpc);
criterion_main!(noop_tarpc);
