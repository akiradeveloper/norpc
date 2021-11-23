# Hello-World Example

Welcome to norpc!

This is a tutorial for new comers.
Understading the minimal hello-world example
gives you an ability to write any extensive applications.

## (1) Define your service

First, you have to define your service.
The service is a set of functions.
You don't need to have `Result` type in the output 
because you can give your application a error type in later section.

```
service HelloWorld {
    fn hello(s: String) -> String;
}
```

## (2) Generate the code

norpc compiler generates a Rust code from the service definition.
You can use `norpc::build::Compiler` in build.rs to generate the code 
in build time.

```rust
fn main() {
    norpc::build::Compiler::new().compile("hello_world.norpc");
}
```

Don't forget to add build-dependencies to Cargo.toml.

```
[build-dependencies]
norpc = "0.3"
```

## (3) Include the code

The generated code is placed in somewhere under the target directory.
To include the code, you can use `norpc::include_code!` macro.
The code is truly included just like C include.

```rust
norpc::include_code!("hello_world");
```

## (4) Implement the application

Trait definition is included in the generated code and you have to implement the trait. The recommendation here is to name it `struct ServiceNameApp`.

You can define your own error type here and let the functions returns the error.
This error will be propagated back to the client throughout the channel.

```rust
#[derive(Clone)]
struct HelloWorldApp;
#[norpc::async_trait]
impl HelloWorld for HelloWorldApp {
    type Error = ();
    async fn hello(self, s: String) -> Result<String, Self::Error> {
        Ok(format!("Hello, {}", s))
    }
}
```

Note that this struct should be `Clone` because the application is cloned in concurrent requests. So if your application have shared state, wrap it in `Arc` to share the state between threads.

## (5) Start the server on async runtime

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

## (6) Access the server from the client

To access the server from the client, use the `Sender` side of the channel.

```rust
    let chan = norpc::ClientChannel::new(tx);
    let mut cli = HelloWorldClient::new(chan); // HelloWorldClient is auto-generated
    assert_eq!(cli.hello("World".to_owned()).await.unwrap(), "Hello, World");
```

The client is auto-generated and each function has return type of `Result<T, norpc::Error<HelloWorldApp::Error>>`.

The `norpc::Error` is an enum of application error and the transport error so you can know on error what's happened under the function call.

```rust
pub enum Error<AppError> {
    AppError(AppError),
    TransportError(TransportError),
}
```