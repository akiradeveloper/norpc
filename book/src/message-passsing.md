# Message Passing

To share state between async processes, these two possible solutions can be considered

1. Shared memory
2. Message passing

The benefit of message passing is
the processes are isolated and only communicated using defined messages.
Each process typically holds some resources like storage or connection to external service as a sole owner
and encapsulates the direct access to the resource from other processes.
This makes developing async applications easy because your interest is minimized.

You can also read this documentation from Tokio.
[https://tokio.rs/tokio/tutorial/channels](https://tokio.rs/tokio/tutorial/channels)

## Problem: Boilarplate

Let's design your async application by message passing.
In this case, you have to define your own message types for request and response by hand
and may have to write some logics that consumes messages from channel or send response to the sender
using oneshot channel. From the Tokio documentation this could be like this:

```rust
use tokio::sync::oneshot;
use bytes::Bytes;

/// Multiple different commands are multiplexed over a single channel.
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;
```

```rust
while let Some(cmd) = rx.recv().await {
    match cmd {
        Command::Get { key, resp } => {
            let res = client.get(&key).await;
            // Ignore errors
            let _ = resp.send(res);
        }
        Command::Set { key, val, resp } => {
            let res = client.set(&key, val).await;
            // Ignore errors
            let _ = resp.send(res);
        }
    }
}
```

However, writing such codes is really tedious.

## Solution: Code generation

The solution is to generate code so you can 
focus on the logics rather than the boilarplates.

With norpc, you can define your in-memory microservice
like this and this will generate all the other tedious codes.

```rust
#[norpc::service]
trait YourService {
    fn get(key: String) -> Option<Bytes>;
    fn set(key: String, val: Bytes);
}
```