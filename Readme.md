# Wait-free synchronization primitives

[![CI](https://github.com/novomation/waitfree-sync/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/novomation/waitfree-sync/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](
https://github.com/novomation/waitfree-sync#license)

Wait-freedom is the strongest non-blocking guarantee, ensuring that every thread completes its operations within a bounded number of steps.
This library provides a collection of wait-free algorithms.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
waitfree-sync = " ... "
```

## Triple Buffer

A wait-free triple buffer for single-producer, single-consumer scenarios.

```rust
use waitfree_sync::triple_buffer;

fn main() {
    let (mut wr, mut rd) = triple_buffer::triple_buffer();

    wr.write(42);
    assert_eq!(wr.try_read(), Some(42));
    assert_eq!(rd.try_read(), Some(42));
}
```

## SPSC Queue

A wait-free single-producer, single-consumer queue.

```rust
use waitfree_sync::spsc;

fn main() {
    let (tx, rx) = spsc::spsc(8);

    tx.try_send("hello").unwrap();
    assert_eq!(rx.try_recv(), Some("hello"));
}
```

## Features

- **No locks:** All operations are wait-free.
- **No dynamic allocation:** All memory is allocated up front.
- **Suitable for real-time systems:** Progress is guaranteed for every thread.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)

## Roadmap

- [ ] Add nostd support
- [ ] Add MPSC/SPMC/MPMC queues