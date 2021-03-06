# Runtime

![](norpc-stack.png)

The yellow and blue parts are called "Runtime".

As mentioned in the earlier section,
the code generated by the compiler (in red) are runtime-agnostic.

To run the service in a specific runtime, you need to write some code.

Firstly, you need to implement the generated application trait.

```rust
struct HelloWorldApp;
#[async_trait::async_trait]
impl HelloWorld for HelloWorldApp {
    async fn hello(&self, s: String) -> String {
        format!("Hello, {}", s)
    }
}
```

Then, build a server and a connecting channel.
After spawning the server in a async runtime, you can
send requests to the server through the channel.

```rust
use norpc::runtime::tokio::*;

let app = HelloWorldApp;
let builder = ServerBuilder::new(HelloWorldService::new(app));
let (chan, server) = builder.build();

tokio::spawn(server.serve());

let mut cli = HelloWorldClient::new(chan);
assert_eq!(cli.hello("World".to_owned()).await, "Hello, World");
```