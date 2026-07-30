# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.4.0] - 2026-07-30

### Added

- Add an MSRV check to CI
- Detect when the other half of the SPSC queue has been dropped: `Sender::try_send` returns `SendError::ReceiverSideDropped` and `Receiver::try_recv` returns `TryRecvError::Disconnected`.

### Changed

- [**breaking**] Replace `NoSpaceLeftError` with a `SendError` enum with `NoSpaceLeft` and `ReceiverSideDropped` variants, returned by `Sender::try_send`
- [**breaking**] `Receiver::try_recv` now returns `Result<T, TryRecvError>` instead of `Option<T>`, distinguishing an empty queue (`TryRecvError::Empty`) from a disconnected `Sender` (`TryRecvError::Disconnected`)
- Bump dependencies

### Fixed

- Fix Miri and Loom tests to correctly account for the drop behavior of the SPSC queue

## [0.3.3] - 2026-05-18

### Added

- Add `len()` and `is_empty()` methods to the SPSC queue
- Add SPSC benchmarks

### Changed

- Raise the minimum supported Rust version (MSRV) to 1.81
- Replace the `affinity` dev-dependency with `core_affinity` to fix minimal-versions builds

## [0.3.2] - 2025-10-21

### Changed

- Add badges to the README
- Improve documentation

## [0.3.1] - 2025-10-10

### Changed

- Improve documentation
- Fix a missing `mut` in an example
- Include the README in the crate metadata
- Exclude the `.github` directory from the published crate

## [0.3.0] - 2025-10-09

### Fixed

- [**breaking**] Rename `size()` to `capacity()` on `Sender` and `Receiver`

## [0.2.1] - 2025-10-07

### Added

- Clean up the internal structure and remove an unneeded raw pointer
- Add a `peek` method to the SPSC receiver

### Fixed

- Remove an unnecessary `Sized` trait bound
- Correctly drop the old value in the triple buffer under loom
- Fix remaining loom test failures

## [0.2.0] - 2025-09-12

### Added

- Add a `read()` method to the triple buffer writer
- Add a size option for the SPSC queue
- [**breaking**] Provide the SPSC capacity as a function parameter instead of a const generic

### Changed

- Add more documentation
- Rename methods to match the standard library channel API

### Fixed

- Fix a memory leak caused by `Option` inside `UnsafeCell`
- Remove an unnecessary write guard
- Fix a missing `Some()` wrapping under loom
- Fix test naming for loom
- Fix a failing loom test
- Fix clippy lints
- Fix missing renames in loom code and documentation
- Fix documentation errors
