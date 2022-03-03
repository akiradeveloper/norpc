# Runtime

![](norpc-stack.png)

The yellow and blue parts are called "Runtime".

As mentioned in the earlier section,
the code generated by the compiler (in red) are runtime-agnostic.

To run the service in a specific runtime, you need to write some code.

Firstly, you need to implement the generated application trait.
Note that the application must implement `Clone` and the cloning is assumed to be cheap.
When you have a state within the application you can use `Arc` to share the reference between threads.

```rust
#[derive(Clone)]
struct HelloWorldApp;
#[norpc::async_trait]
impl HelloWorld for HelloWorldApp {
    async fn hello(self, s: String) -> String {
        format!("Hello, {}", s)
    }
}
```

Secondly, start a server with a receiver-side of a channel.
You can connect to the server by sender-side of the channel and
the client that consumes it.

```rust
use norpc::runtime::send::*;
let (tx, rx) = mpsc::channel(100);
tokio::spawn(async move {
	let app = HelloWorldApp;
	let service = HelloWorldService::new(app);
	let server = ServerExecutor::new(rx, service);
	server.serve().await
});
let chan = ClientService::new(tx);
let mut cli = HelloWorldClient::new(chan);
assert_eq!(cli.hello("World".to_owned()).await.unwrap(), "Hello, World");
```