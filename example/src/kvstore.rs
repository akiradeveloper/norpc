use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

#[norpc::service]
trait KVStore {
    fn read(id: u64) -> Option<std::string::String>;
    fn write(id: u64, s: String) -> ();
    fn write_many(kv: std::collections::HashSet<(u64, std::string::String)>);
    fn list() -> Vec<(u64, std::string::String)>;
    fn ret_any_tuple() -> (u8, u8);
    // We can return a result from app to the client.
    fn noop() -> std::result::Result<bool, ()>;
}

struct KVStoreApp {
    state: RwLock<HashMap<u64, String>>,
}
impl KVStoreApp {
    fn new() -> Self {
        Self {
            state: RwLock::new(HashMap::new()),
        }
    }
}
#[norpc::async_trait]
impl KVStore for KVStoreApp {
    async fn read(&self, id: u64) -> Option<String> {
        self.state.read().await.get(&id).cloned()
    }
    async fn write(&self, id: u64, v: String) {
        self.state.write().await.insert(id, v);
    }
    async fn write_many(&self, kv: HashSet<(u64, String)>) {
        for (k, v) in kv {
            self.state.write().await.insert(k, v);
        }
    }
    async fn list(&self) -> Vec<(u64, String)> {
        let mut out = vec![];
        let reader = self.state.read().await;
        for (k, v) in reader.iter() {
            out.push((*k, v.clone()));
        }
        out
    }
    async fn ret_any_tuple(&self) -> (u8, u8) {
        (0, 0)
    }
    async fn noop(&self) -> Result<bool, ()> {
        Ok(true)
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_kvstore() {
    use norpc::runtime::*;

    let app = KVStoreApp::new();
    let service = KVStoreService::new(app);
    let (chan, server) = ServerBuilder::new(service).build();
    ::tokio::spawn(server.serve(TokioExecutor));

    let mut cli = KVStoreClient::new(chan);
    assert_eq!(cli.read(1).await, None);
    cli.write(1, "one".to_owned()).await;
    assert_eq!(cli.read(1).await, Some("one".to_owned()));
    assert_eq!(cli.read(2).await, None);
    assert_eq!(cli.read(3).await, None);

    let mut cli2 = cli.clone();
    let mut h = HashSet::new();
    h.insert((2, "two".to_owned()));
    h.insert((3, "three".to_owned()));
    cli2.write_many(h).await;
    assert_eq!(cli2.read(3).await, Some("three".to_owned()));
    assert_eq!(cli2.noop().await, Ok(true));
}
