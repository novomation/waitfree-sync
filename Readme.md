# Wait-free synchronization primitives

[![CI](https://github.com/novomation/waitfree-sync/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/novomation/waitfree-sync/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](
https://github.com/novomation/waitfree-sync#license)

Wait-freedom is the strongest non-blocking guarantee, ensuring that every thread completes its operations within a bounded number of steps. Lock-freedom only guarantees system-wide progress.

Wait-free synchronization primitives:
* Do not allocate memory on demand
* Are independent of scheduling
* Are a good fit for real-time systems

This library provides a collection of wait-free algorithms.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
waitfree-sync = "0.1"
```

## Examples

### Triple Buffer

A wait-free triple buffer for single-producer, single-consumer scenarios:

```rust
use waitfree_sync::triple_buffer::triple_buffer;

fn main() {
    let (mut writer, mut reader) = triple_buffer();

    writer.write(42);
    assert_eq!(writer.read(), Some(42));
    assert_eq!(reader.read(), Some(42));
}
```

### SPSC Queue

A wait-free single-producer, single-consumer queue:

```rust
use waitfree_sync::spsc::spsc;

fn main() {
    let (mut writer, mut reader) = spsc::<_, 4>();

    writer.write("hello").unwrap();
    assert_eq!(reader.read(), Some("hello"));
}
```

## Features

- **No dynamic allocation:** All memory is allocated up front.
- **No locks:** All operations are wait-free.
- **Suitable for real-time systems:** Progress is guaranteed for every thread.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)

## Readmap

- [ ] Add nostd support
- [ ] Add MPSC/SPMC/MPMC queues