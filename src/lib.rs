#[cfg(loom)]
mod import {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    pub(crate) use loom::sync::Arc;
}

#[cfg(not(loom))]
mod import {
    pub(crate) use core::cell::UnsafeCell;
    pub(crate) use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    pub(crate) use std::sync::Arc;
}

pub mod spsc;
pub mod triple_buffer;
