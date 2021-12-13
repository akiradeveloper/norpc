# Motivation

Developing an async application is often a very difficult task but
building an async application as a set of microservices makes both designing and implementation much easier.

gRPC is a great tool in microservices. You can use this for communication over network but this isn't a good idea unless networking involves.

In such case, in-process microservices is a way to go. The services run on async runtime and communicate each other through in-memory async channel which doesn't occur serialization thus much more efficient than gRPC.
I believe in-process microservices is a revolution for designing local async applications.

However, defining microservices in Rust does need a lot of coding for each services and they are mostly boilerplates. It will be helpful if these tedious tasks are swiped away by code generation.

[google/tarpc](https://github.com/google/tarpc) is a previous work in this area however it is not a best framework for in-process microservices because it tries to support both in-process and networking microservices under the same abstraction. This isn't a good idea because the both implementation will because sub-optimal. In my opinion, networking microservices should use gRPC and in-process microservices should use dedicated framework for the specific purpose.

Also, tarpc doesn't use Tower's `Service` but define a similar abstraction called `Serve` by itself. This leads to reimplementing functions like rate-limiting and timeout which can be realized by just stacking `Service` decorators if depends on Tower. Since tarpc needs huge rework to become Tower-based, there is a chance to implement my own framework from scratch which will be much smaller and cleaner than tarpc because it only supports in-process microservices and is able to exploit the Tower ecosystem.