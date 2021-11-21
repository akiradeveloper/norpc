use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tower::Service;

norpc::include_code!("concurrent_message");

#[derive(Clone)]
struct IdAllocApp {
    n: Arc<AtomicU64>,
    id_store_cli: IdStoreClientT,
}
impl IdAllocApp {
    fn new(id_store_cli: IdStoreClientT) -> Self {
        Self {
            n: Arc::new(AtomicU64::new(1)),
            id_store_cli,
        }
    }
}
#[norpc::async_trait]
impl IdAlloc for IdAllocApp {
    type Error = ();
    async fn alloc(mut self, name: u64) -> Result<u64, Self::Error> {
        let sleep_time = rand::random::<u64>() % 100;
        tokio::time::sleep(std::time::Duration::from_millis(sleep_time)).await;
        let id = self.n.fetch_add(1, Ordering::SeqCst);
        self.id_store_cli.save(name, id).await.unwrap();
        Ok(name)
    }
}

#[derive(Clone)]
struct IdStoreApp {
    map: Arc<RwLock<HashMap<u64, u64>>>,
}
impl IdStoreApp {
    fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
#[norpc::async_trait]
impl IdStore for IdStoreApp {
    type Error = ();
    async fn save(self, name: u64, id: u64) -> Result<(), Self::Error> {
        self.map.write().await.insert(name, id);
        Ok(())
    }
    async fn query(self, name: u64) -> Result<Option<u64>, Self::Error> {
        let id0 = self.map.read().await.get(&name).cloned();
        Ok(id0)
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_message() {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let app = IdStoreApp::new();
        let service = IdStoreService::new(app);
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let mut id_store_cli = IdStoreClient::new(chan);

    let (tx, rx) = mpsc::unbounded_channel();
    let id_store_cli_cln = id_store_cli.clone();
    tokio::spawn(async move {
        let app = IdAllocApp::new(id_store_cli_cln);
        let service = IdAllocService::new(app);
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let id_alloc_cli = IdAllocClient::new(chan);

    let mut queue = futures::stream::FuturesUnordered::new();
    for i in 1..=10000 {
        let mut cli = id_alloc_cli.clone();
        let fut = async move { cli.alloc(i).await };
        queue.push(fut);
    }
    use futures::StreamExt;
    let mut diff_cnt = 0;
    while let Some(Ok(name)) = queue.next().await {
        let id0 = id_store_cli.query(name).await.unwrap();
        assert!(id0.is_some());
        let id = id0.unwrap();
        if id != name {
            diff_cnt += 1;
        }
    }
    assert!(diff_cnt > 0);
}
