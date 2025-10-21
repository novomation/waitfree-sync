# Wait-Free Synchronization Primitives for Fearless Real-Time in Rust

[![Crates.io](https://img.shields.io/crates/v/waitfree_sync.svg)](https://crates.io/crates/waitfree_sync)
[![Documentation](https://docs.rs/waitfree-sync/badge.svg)](
https://docs.rs/waitfree_sync)
[![CI](https://github.com/novomation/waitfree-sync/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/novomation/waitfree-sync/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](
https://github.com/novomation/waitfree-sync#license)

Wait-freedom is the strongest non-blocking guarantee, ensuring that each function completes its operations in a finite number of steps.
This library offers a variety of wait-free algorithms that enable fearless real-time systems.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
waitfree-sync = " ... "
```

## Triple Buffer

A wait-free triple buffer for single-producer, single-consumer scenarios.
This allows the `Reader` to always read the latest update from the `Writer`.
They can both live in different threads with different execution intervals.

```rust
use waitfree_sync::triple_buffer;

let (mut wr, mut rd) = triple_buffer::triple_buffer();

wr.write(42);
assert_eq!(wr.try_read(), Some(42));
assert_eq!(rd.try_read(), Some(42));
```

## SPSC Queue

A wait-free single-producer, single-consumer queue to pass data from one to another thread.
This is useful when passing ordered data where every value is important.

```rust
use waitfree_sync::spsc;

let (mut tx, mut rx) = spsc::spsc(8);

tx.try_send("hello").unwrap();
assert_eq!(rx.try_recv(), Some("hello"));
```

## Features

- **No locks:** All operations are wait-free.
- **No dynamic allocation:** All memory is allocated up front.
- **Suitable for real-time systems:** Progress is guaranteed for every function.

## Background 

The fundamental problem with interprocess communication is maintaining data consistency
when multiple processes can write to and read from a shared memory area simultaneously.
There are three main types of synchronization mechanisms for maintaining data consistency: lock-based, lock-free, and wait-free.

Lock-based mechanisms regulate access to a shared resource through exclusive and therefore blocking access.
A representative of this type is the [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) from the Rust standard library.
Typical challenges include deadlocks, process starvation, and priority inversion. For this reason, no guarantees can be made regarding execution.
Nevertheless, mutexes are widely used due to their simple principle and can achieve sufficient throughput in non-real-time applications.

Lock-free synchronization mechanisms ensure that at least one process will progress within a finite number of steps.
However, process starvation can still occur. The standard library provides [mpsc](https://doc.rust-lang.org/std/sync/mpsc/index.html)
and [mpmc](https://doc.rust-lang.org/std/sync/mpmc/index.html) queues for this purpose.
Lock-free queues are widely used to connect asynchronous processes with high throughput. However, both lock-based and lock-free mechanisms depend on scheduling.

A wait-free system guarantees that all processes can make progress in a finite number of steps, regardless of the state of the other processes.
In other words, processes never have to wait for another process to make progress. Therefore, wait-free methods are independent of scheduling.
Incorrect communication configurations and the side effects of non-real-time to real-time communication are eliminated.
Therefore, data channels with wait-free behavior are very attractive for implementing real-time systems.

The SPSC queue in this crate is based on the improved FastForward queue described in
V. Maffione, G. Lettieri, und L. Rizzo, **„Cache-aware design of general-purpose Single-Producer–Single-Consumer queues“**, Software: Practice and Experience, Bd. 49, Nr. 5, S. 748–779, 2019, doi: [10.1002/spe.2675](https://doi.org/10.1002/spe.2675).
The concept of the triple buffer can be found in various C/C++ and Rust implementations, such as [here](https://github.com/HadrienG2/triple-buffer).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)

## Roadmap

- [ ] Add nostd support
- [ ] Add MPSC/SPMC/MPMC queues