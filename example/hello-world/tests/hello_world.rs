use tokio::sync::mpsc;
use tower::Service;
use std::rc::Rc;

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

#[norpc::service(?Send)]
trait HelloWorldLocal {
    // Rc<T> is !Send
    fn hello(s: Rc<String>) -> Rc<String>;
}
#[derive(Clone)]
struct HelloWorldLocalApp;
#[norpc::async_trait(?Send)]
impl HelloWorldLocal for HelloWorldLocalApp {
    async fn hello(self, s: Rc<String>) -> Rc<String> {
        format!("Hello, {}", s).into()
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_hello_world_no_send() {
    let local = tokio::task::LocalSet::new();
    let (tx, rx) = mpsc::unbounded_channel();
    local.spawn_local(async move {
        let app = HelloWorldLocalApp;
        let service = HelloWorldLocalService::new(app);
        let server = norpc::no_send::ServerChannel::new(rx, service);
        server.serve().await
    });
    local.spawn_local(async move {
        let chan = norpc::no_send::ClientChannel::new(tx);
        let mut cli = HelloWorldLocalClient::new(chan);
        assert_eq!(
            cli.hello("World".to_owned().into()).await.unwrap(),
            "Hello, World".to_string().into()
        );
    });
    local.await;
}