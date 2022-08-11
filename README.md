# norpc = not remote procedure call

[![Crates.io](https://img.shields.io/crates/v/norpc.svg)](https://crates.io/crates/norpc)
[![documentation](https://docs.rs/norpc/badge.svg)](https://docs.rs/norpc)
![CI](https://github.com/akiradeveloper/norpc/workflows/CI/badge.svg)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/akiradeveloper/norpc/blob/master/LICENSE)
[![Tokei](https://tokei.rs/b1/github/akiradeveloper/norpc)](https://github.com/akiradeveloper/norpc)

[Documentation](https://akiradeveloper.github.io/norpc/)

## Example

```rust
#[norpc::service]
trait HelloWorld {
   fn hello(s: String) -> String;
}
struct HelloWorldApp;
#[async_trait::async_trait]
impl HelloWorld for HelloWorldApp {
   async fn hello(&self, s: String) -> String {
       format!("Hello, {}", s)
   }
}
let rep = tokio_test::block_on(async {
    use norpc::runtime::*;
    let app = HelloWorldApp;
    let svc = HelloWorldService::new(app);
    let (chan, server) = ServerBuilder::new(svc).build();
    tokio::spawn(server.serve(TokioExecutor));
    let mut cli = HelloWorldClient::new(chan);
    cli.hello("World".to_owned()).await
});
assert_eq!(rep, "Hello, World");
```

## Usage

```
norpc = { version = "0.9", features = ["runtime", "tokio-executor"] }
```

- runtime: Use norpc runtime
- tokio-executor: Use tokio as async runtime.
- async-std-executor: Use async-std as async runtime.

## Features

- Support in-process microservices through async channel.
- Async runtime agnostic.
- Support non-`Send` types.
- Support request cancellation from client.

## Performance

norpc is about 2x faster than [google/tarpc](https://github.com/google/tarpc).

To compare the pure overhead, he benchmark program launches
a no-op server and send requests from the client.

```
noop request/1          time:   [8.9181 us 8.9571 us 9.0167 us]
noop request (tarpc)/1  time:   [15.476 us 15.514 us 15.554 us]
```

## Author

Akira Hayakawa (@akiradeveloper)
