# norpc = not remote procedure call

[![Crates.io](https://img.shields.io/crates/v/norpc.svg)](https://crates.io/crates/norpc)
[![documentation](https://docs.rs/norpc/badge.svg)](https://docs.rs/norpc)
![CI](https://github.com/akiradeveloper/norpc/workflows/CI/badge.svg)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/akiradeveloper/norpc/blob/master/LICENSE)
[![Tokei](https://tokei.rs/b1/github/akiradeveloper/norpc)](https://github.com/akiradeveloper/norpc)

[Documentation](https://akiradeveloper.github.io/norpc/)

```rust
#[norpc::service]
trait HelloWorld {
    fn hello(s: String) -> String;
}
```

## Features

- Support in-process microservices through async channel.
- Support non-`Send` types.

## Performance

The RPC overhead is x1.7 lower than [google/tarpc](https://github.com/google/tarpc). With norpc, you can send more than 100k requests per second.

The benchmark program launches a noop server and send requests from the client.
In measurement, [Criterion](https://github.com/bheisler/criterion.rs) is used.

```
noop request/1          time:   [8.9181 us 8.9571 us 9.0167 us]
noop request (tarpc)/1  time:   [15.476 us 15.514 us 15.554 us]
```

## Author

Akira Hayakawa (@akiradeveloper)
