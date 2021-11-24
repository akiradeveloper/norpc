use tokio::sync::mpsc;
use tower::Service;

#[norpc::service]
trait HelloWorld {
    fn hello(s: String) -> String;
}

#[derive(Clone)]
struct HelloWorldApp;
#[norpc::async_trait]
impl HelloWorld for HelloWorldApp {
    async fn hello(self, s: String) -> String {
        format!("Hello, {}", s)
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_hello_world() {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let app = HelloWorldApp;
        let service = HelloWorldService::new(app);
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let mut cli = HelloWorldClient::new(chan);
    assert_eq!(cli.hello("World".to_owned()).await.unwrap(), "Hello, World");
}
