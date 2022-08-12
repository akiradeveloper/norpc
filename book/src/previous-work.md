# Previous Work

[Tarpc](https://github.com/google/tarpc) is a previous work in this area.

You can define your service by trait and the macro `tarpc::service` generates all the rest of the codes.

```rust
#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
}
```

## Problems

However, I found two problems in Tarpc.

### 1. Tarpc isn't optimized for in-memory channel

The goal of tarpc is providing RPC through TCP channel
and the direct competitor is RPC framework like [Tonic](https://github.com/hyperium/tonic)
or [jsonrpc](https://github.com/paritytech/jsonrpc).

Tarpc only allows to use in-memory channel under the same abstraction
so the implementation isn't optimized for in-memory channel.

### 2. Tarpc doesn't use Tower

[Tower](https://github.com/tower-rs/tower) is a framework like
Scala's [Finagle](https://twitter.github.io/finagle/)
which provides a abstraction called `Service` which is like a function from request to response
and decorator stacks to add more functionality on top of the abstraction.

If we design a RPC framework from scratch with the current Rust ecosystem,
we will 100% choose to depend on Tower to implement
functionalities like rate-limiting or timeout which is essential in doing RPC.
In fact, Tonic does so.

However, Tarpc's started a long ago before the current Rust ecosystem is established
and it doesn't use Tower but implements those functionalities by itself.