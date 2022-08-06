#[norpc::service]
trait Panic {
    fn panic();
}
struct App;
#[norpc::async_trait]
impl Panic for App {
    async fn panic(&self) {
        panic!("I am panicked!");
    }
}
#[tokio::test]
#[should_panic]
async fn test_panic() {
    use norpc::runtime::*;

    let app = App;
    let service = PanicService::new(app);
    let (chan, server) = ServerBuilder::new(service).build();
    ::tokio::spawn(server.serve(tokio::TokioExecutor));

    let mut cli = PanicClient::new(chan);
    cli.panic().await;
}
