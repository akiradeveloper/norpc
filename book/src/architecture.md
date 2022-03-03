# Architecture

![](images/norpc-stack.png)

norpc utilizes Tower ecosystem.

The core of the Tower ecosystem is an abstraction called `Service` which is like a function from `Request` to `Response`.
The ecosystem has many decorator stacks to add new behavior to an existing `Service`.

In the diagram, the client requests is coming from the top-left of the stacks and flows down to the bottom-right.
The client and server is connected by async channel driven by Tokio runtime so there is no overhead for the serialization
and copying because the message just "moves".

