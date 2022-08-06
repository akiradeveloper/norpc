use std::rc::Rc;

#[norpc::service]
trait HelloWorld {
    fn hello(s: String) -> String;
}
struct HelloWorldApp;
#[norpc::async_trait]
impl HelloWorld for HelloWorldApp {
    async fn hello(&self, s: String) -> String {
        format!("Hello, {}", s)
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_hello_world() {
    use norpc::runtime::*;
    let app = HelloWorldApp;
    let builder = ServerBuilder::new(HelloWorldService::new(app));
    let (chan, server) = builder.build();
    ::tokio::spawn(server.serve(tokio::TokioExecutor));
    let mut cli = HelloWorldClient::new(chan);
    assert_eq!(cli.hello("World".to_owned()).await, "Hello, World");
}

#[norpc::service(?Send)]
trait HelloWorldLocal {
    // Rc<T> is !Send
    fn hello(s: Rc<String>) -> Rc<String>;
}
struct HelloWorldLocalApp;
#[norpc::async_trait(?Send)]
impl HelloWorldLocal for HelloWorldLocalApp {
    async fn hello(&self, s: Rc<String>) -> Rc<String> {
        format!("Hello, {}", s).into()
    }
}
