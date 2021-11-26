use std::rc::Rc;
use tokio::sync::mpsc;
use tower::Service;

#[norpc::service(?Send)]
trait HelloWorld {
    // Rc<T> is !Send
    fn hello(s: Rc<String>) -> Rc<String>;
}

#[derive(Clone)]
struct HelloWorldApp;
#[norpc::async_trait(?Send)]
impl HelloWorld for HelloWorldApp {
    async fn hello(self, s: Rc<String>) -> Rc<String> {
        format!("Hello, {}", s).into()
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_hello_world_no_send() {
    let local = tokio::task::LocalSet::new();
    let (tx, rx) = mpsc::unbounded_channel();
    local.spawn_local(async move {
        let app = HelloWorldApp;
        let service = HelloWorldService::new(app);
        let server = norpc::no_send::ServerChannel::new(rx, service);
        server.serve().await
    });
    local.spawn_local(async move {
        let chan = norpc::no_send::ClientChannel::new(tx);
        let mut cli = HelloWorldClient::new(chan);
        assert_eq!(
            cli.hello("World".to_owned().into()).await.unwrap(),
            "Hello, World".to_string().into()
        );
    });
    local.await;
}
