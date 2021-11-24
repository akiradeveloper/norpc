# Hello-World Example

Welcome to norpc!

This is a tutorial for new comers.
Understading the minimal hello-world example
gives you an ability to write any extensive applications.

## (1) Define your service

First, you have to define your service. The service is a set of functions.

```
#[norpc::service]
trait HelloWorld {
    fn hello(s: String) -> String;
}
```

## (2) Implement the application

Trait definition is included in the generated code and you have to implement the trait.

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

Note that this struct should be `Clone` because the application is cloned in concurrent requests.
So if your application have shared state, wrap it in `Arc` to share the state between threads.

## (3) Start the server on Tokio runtime

Now, definition phase is over. Let's start the server in your main function.

First, create a `mpsc::unbounded_channel` to connect the server and clients.
Then, feed the `Receiver` side to the server-side and start the event loop.

```rust
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let app = HelloWorldApp;
        let service = HelloWorldService::new(app); // HelloWorldService is auto-generated
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });

```

## (4) Access the server from the client

To access the server from a client, use the `Sender` side of the channel.

```rust
    let chan = norpc::ClientChannel::new(tx);
    let mut cli = HelloWorldClient::new(chan); // HelloWorldClient is auto-generated
    assert_eq!(cli.hello("World".to_owned()).await.unwrap(), "Hello, World");
```

The client is auto-generated and each function has return type `Result<T, norpc::Error>`.