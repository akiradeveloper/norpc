# Example

## Option or Result

You can return `Option` or `Result` types to
propagate some failure back to the client.

```rust
#[norpc::service]
trait YourService {
    fn read(id: u64) -> Option<Bytes>;
    fn write(id: u64, b: Bytes) -> Result<usize, ()>;
}
```

## Non-Send

You can generate non-Send service by add `?Send` parameter to `norpc::service` macro.

This is useful when you want to run the service in pinned thread.
Some runtime requires non-Send type for this reason.

```rust
#[norpc::service(?Send)]
trait YourService {
    // Rc<T> is !Send
    fn echo(s: Rc<String>) -> Rc<String>;
}
```