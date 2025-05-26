#[cfg(loom)]
mod import {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::atomic::{AtomicUsize, Ordering};
    pub(crate) use loom::sync::Arc;
}

#[cfg(not(loom))]
mod import {
    pub(crate) use core::cell::UnsafeCell;
    pub(crate) use std::sync::atomic::{AtomicUsize, Ordering};
    pub(crate) use std::sync::Arc;
}

mod inital_write_guard;
pub mod triple_buffer;
