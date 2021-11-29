# norpc = not remote procedure call

[![Crates.io](https://img.shields.io/crates/v/norpc.svg)](https://crates.io/crates/norpc)
[![documentation](https://docs.rs/norpc/badge.svg)](https://docs.rs/norpc)
![CI](https://github.com/akiradeveloper/norpc/workflows/CI/badge.svg)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/akiradeveloper/norpc/blob/master/LICENSE)
[![Tokei](https://tokei.rs/b1/github/akiradeveloper/norpc)](https://github.com/akiradeveloper/norpc)

## Motivation

Developing an async application is often a very difficult task but
building an async application as a set of microservices makes both designing and implementation much easier.

gRPC is a great tool in microservices. You can use this for communication over network but this isn't a good idea unless networking involves.

In such case, in-process microservices is a way to go. The services run on async runtime and communicate each other through in-memory async channel which doesn't occur serialization thus much more efficient than gRPC.
I believe in-process microservices is a revolution for designing local async applications.

However, defining microservices in Rust does need a lot of coding for each services and they are mostly boilerplates. It will be helpful if these tedious tasks are swiped away by code generation.

[tarpc](https://github.com/google/tarpc) is a previous work in this area however it is not a best framework for in-process microservices because it tries to support both in-process and networking microservices under the same abstraction. This isn't a good idea because the both implementation will because sub-optimal. In my opinion, networking microservices should use gRPC and in-process microservices should use dedicated framework for the specific purpose.

Also, tarpc doesn't use Tower's `Service` but define a similar abstraction called `Serve` by itself. This leads to reimplementing functions like rate-limiting and timeout which can be realized by just stacking `Service` decorators if depends on Tower. Since tarpc needs huge rework to become Tower-based, there is a chance to implement my own framework from scratch which will be much smaller and cleaner than tarpc because it only supports in-process microservices and is able to exploit the Tower ecosystem.

## Architecture

![スクリーンショット 2021-11-30 8 47 23](https://user-images.githubusercontent.com/785824/143960488-75d9d959-f763-4978-bc51-82fe159cd8ad.png)

**norpc utilizes Tower ecosystem.**
The core of the Tower ecosystem is an abstraction called `Service` which is like a function from `Request` to `Response`.
The ecosystem has many decorators to add new behavior to an existing `Service`.

In the diagram, the client requests is coming from the top-left of the stacks and flows down to the bottom-right.
The client and server is connected by async channel driven by Tokio runtime so there is no overhead for the serialization
and copying because the message just "moves".

Here is how to generate codes for a simple service:

```rust
#[norpc::service]
trait HelloWorld {
    fn hello(s: String) -> String;
}
```

For more detail, please read [Hello-World Example (README)](example/hello-world/README.md).

## Performance (Compared to tarpc)

The RPC overhead is x1.7 lower than tarpc. With norpc, you can send more than 100k requests per second.

The benchmark program launches a noop server and send requests from the client.
In measurement, [Criterion](https://github.com/bheisler/criterion.rs) is used.

```
noop request/1          time:   [8.9181 us 8.9571 us 9.0167 us]
noop request (tarpc)/1  time:   [15.476 us 15.514 us 15.554 us]
```

## Author

Akira Hayakawa (@akiradeveloper)
