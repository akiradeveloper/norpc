use tokio::sync::mpsc;
use tower::Service;

norpc::include_code!("hello_world");

#[derive(Clone)]
struct HelloWorldApp;
#[norpc::async_trait]
impl HelloWorld for HelloWorldApp {
    type Error = ();
    async fn hello(self, s: String) -> Result<String, Self::Error> {
        Ok(format!("Hello, {}", s))
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