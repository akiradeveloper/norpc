use std::rc::Rc;
use tokio::sync::mpsc;

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
    use norpc::runtime::tokio::*;
    let app = HelloWorldApp;
    let builder = ServerBuilder::new(HelloWorldService::new(app));
    let (chan, server) = builder.build();
    tokio::spawn(server.serve());
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
