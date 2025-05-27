# Wait-free synchronization primitives


Wait-freedom is the strongest non-blocking gurantee, ensuring that every thread completes its operations within a bounded number of steps. Lock-freedom only gurantees system wide progress.

Wait-free synchronization primitvews:
* Do not allocated memory on demand
* Are independant of scheduling.
* Are a good fot for real-time systems.

This library provides a collection of wait-free algorithms.

```Rust

use libr

```