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
#[test]
fn test_async_std_runtime() {
	async_std::task::block_on(async {
		use norpc::runtime::*;
		let app = HelloWorldApp;
		let builder = ServerBuilder::new(HelloWorldService::new(app));
		let (chan, server) = builder.build();
		::async_std::task::spawn(server.serve(async_std::AsyncStdExecutor));
		let mut cli = HelloWorldClient::new(chan);
		assert_eq!(cli.hello("World".to_owned()).await, "Hello, World");
	})
}