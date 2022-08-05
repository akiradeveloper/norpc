/// This test is to prove the the code generation is self-contained.

#[norpc::service]
trait Add {
    fn add(x: u64, y: u64) -> u64;
}
struct AddApp;
#[norpc::async_trait]
impl Add for AddApp {
    async fn add(&self, x: u64, y: u64) -> u64 {
        x + y
    }
}
#[tokio::test]
async fn test_no_runtime() {
    let mut cli = AddClient::new(AddService::new(AddApp));
    let r = cli.add(1, 2).await;
    assert_eq!(r, 3);
}
