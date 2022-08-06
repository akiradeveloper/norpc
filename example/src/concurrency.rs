use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use tokio::sync::RwLock;
use tower::ServiceBuilder;

#[norpc::service]
trait IdAlloc {
    fn alloc(name: u64) -> u64;
}
#[norpc::service]
trait IdStore {
    fn save(name: u64, id: u64);
    fn query(name: u64) -> Option<u64>;
}

type IdStoreClientService = norpc::runtime::Channel<IdStoreRequest, IdStoreResponse>;
type IdStoreClientT = IdStoreClient<IdStoreClientService>;
struct IdAllocApp {
    n: AtomicU64,
    id_store_cli: IdStoreClientT,
}
impl IdAllocApp {
    fn new(id_store_cli: IdStoreClientT) -> Self {
        Self {
            n: AtomicU64::new(1),
            id_store_cli,
        }
    }
}
#[norpc::async_trait]
impl IdAlloc for IdAllocApp {
    async fn alloc(&self, name: u64) -> u64 {
        let sleep_time = rand::random::<u64>() % 100;
        tokio::time::sleep(std::time::Duration::from_millis(sleep_time)).await;
        let id = self.n.fetch_add(1, Ordering::SeqCst);
        self.id_store_cli.clone().save(name, id).await;
        name
    }
}

struct IdStoreApp {
    map: RwLock<HashMap<u64, u64>>,
}
impl IdStoreApp {
    fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }
}
#[norpc::async_trait]
impl IdStore for IdStoreApp {
    async fn save(&self, name: u64, id: u64) {
        self.map.write().await.insert(name, id);
    }
    async fn query(&self, name: u64) -> Option<u64> {
        self.map.read().await.get(&name).cloned()
    }
}

const N: u64 = 10000;

#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_message() {
    use norpc::runtime::*;

    let app = IdStoreApp::new();
    let service = IdStoreService::new(app);
    let (chan, server) = ServerBuilder::new(service).build();
    ::tokio::spawn(server.serve(tokio::TokioExecutor));
    let mut id_store_cli = IdStoreClient::new(chan);

    let app = IdAllocApp::new(id_store_cli.clone());
    let service = IdAllocService::new(app);
    let service = ServiceBuilder::new()
        // Changing this value will see a different complete time.
        .concurrency_limit(100)
        .service(service);
    let (chan, server) = ServerBuilder::new(service).build();
    ::tokio::spawn(server.serve(tokio::TokioExecutor));
    let id_alloc_cli = IdAllocClient::new(chan);

    let mut queue = futures::stream::FuturesUnordered::new();
    for i in 1..=N {
        let mut cli = id_alloc_cli.clone();
        let fut = async move { cli.alloc(i).await };
        queue.push(fut);
    }
    use futures::StreamExt;
    let mut n = 0;
    let mut diff_cnt = 0;
    while let Some(name) = queue.next().await {
        n += 1;
        let id0 = id_store_cli.query(name).await;
        assert!(id0.is_some());
        let id = id0.unwrap();
        if id != name {
            diff_cnt += 1;
        }
    }
    assert_eq!(n, N);
    assert!(diff_cnt > 0);
}
