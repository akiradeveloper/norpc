use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

#[norpc::service]
trait KVStore {
    fn read(id: u64) -> Option<String>;
    fn write(id: u64, s: String) -> ();
    fn write_many(kv: HashSet<(u64, String)>);
    // We can return a result from app to the client.
    fn noop() -> Result<bool, ()>;
    // If app function fails error is propagated to the client.
    fn panic();
}

#[derive(Clone)]
struct KVStoreApp {
    state: Arc<RwLock<HashMap<u64, String>>>,
}
impl KVStoreApp {
    fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
#[norpc::async_trait]
impl KVStore for KVStoreApp {
    async fn read(self, id: u64) -> Option<String> {
        self.state.read().await.get(&id).cloned()
    }
    async fn write(self, id: u64, v: String) {
        self.state.write().await.insert(id, v);
    }
    async fn write_many(self, kv: HashSet<(u64, String)>) {
        for (k, v) in kv {
            self.state.write().await.insert(k, v);
        }
    }
    async fn noop(self) -> Result<bool, ()> {
        Ok(true)
    }
    async fn panic(self) {
        panic!()
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_kvstore() {
    use norpc::runtime::send::*;
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(async move {
        let app = KVStoreApp::new();
        let service = KVStoreService::new(app);
        let server = ServerExecutor::new(rx, service);
        server.serve().await
    });
    let chan = ClientService::new(tx);

    let mut cli = KVStoreClient::new(chan);
    // It doesn't crash if it fails.
    for _ in 0..10000 {
        assert!(cli.panic().await.is_err());
    }
    assert_eq!(cli.read(1).await.unwrap(), None);
    cli.write(1, "one".to_owned()).await.unwrap();
    assert_eq!(cli.read(1).await.unwrap(), Some("one".to_owned()));
    assert_eq!(cli.read(2).await.unwrap(), None);
    assert_eq!(cli.read(3).await.unwrap(), None);

    let mut cli2 = cli.clone();
    // It doesn't crash if it fails.
    for _ in 0..10000 {
        assert!(cli2.panic().await.is_err());
    }
    let mut h = HashSet::new();
    h.insert((2, "two".to_owned()));
    h.insert((3, "three".to_owned()));
    cli2.write_many(h).await.unwrap();
    assert_eq!(cli2.read(3).await.unwrap(), Some("three".to_owned()));
    assert_eq!(cli2.noop().await.unwrap(), Ok(true));
}
