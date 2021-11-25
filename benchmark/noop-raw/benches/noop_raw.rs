use criterion::{criterion_group, criterion_main, Criterion};

async fn noop() -> Result<(), ()> {
	Ok(())
}

fn bench_noop_raw(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    c.bench_function("noop request (raw)", |b| {
        b.to_async(&rt).iter(|| async {
            noop().await.unwrap();
        });
    });
}

criterion_group!(noop_raw, bench_noop_raw);
criterion_main!(noop_raw);