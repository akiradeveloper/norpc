# Tarpc

[Tarpc](https://github.com/google/tarpc) is a previous work in this area.

You can define your service by trait and the macro `tarpc::service` generates all the rest of the boring codes.

```rust
#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
}
```

However, I found two problems in Tarpc.

## Tarpc isn't optimized for in-memory channel

The goal of tarpc is providing RPC through TCP channel
and the direct competitor is RPC framework like [Tonic](https://github.com/hyperium/tonic)
or [jsonrpc](https://github.com/paritytech/jsonrpc).

Tarpc only allows to use in-memory channel under the same abstraction
so the implementation isn't optimized for in-memory channel.

## Tarpc doesn't use Tower

[Tower](https://github.com/tower-rs/tower) is a framework like
Scala's [Finagle](https://twitter.github.io/finagle/)
which provides a abstraction `Request -> Response` and decorator stacks
to add more functionality to the abstraction.

If we design a RPC framework from scratch with the current Rust ecosystem,
we will 100% choose to depend on Tower to implement
functionalities like rate-limiting or timeout which is essential in doing RPC.
In fact, Tonic does so.

However, Tarpc's started a long ago before the current ecosystem is established
and it doesn't use Tower but implements those functionalities by itself to my surprise.