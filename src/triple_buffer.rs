//! Wait-free single-producer single-consumer (SPSC) triple buffer to share data between two threads.
//!
//! # Example
//! ```rust
//! use waitfree_sync::triple_buffer;
//!
//! let (mut wr, mut rd) = triple_buffer::triple_buffer();
//! wr.write(42);
//! assert_eq!(wr.try_read(), Some(42));
//! assert_eq!(rd.try_read(), Some(42));
//! ```
//!
//!

use crate::import::{Arc, AtomicUsize, Ordering, UnsafeCell};
use crossbeam_utils::CachePadded;

const NEW_DATA_FLAG: usize = 0b100;
const INDEX_MASK: usize = 0b011;

#[derive(Debug)]
struct Shared<T: Sized> {
    mem: [UnsafeCell<Option<T>>; 3],
    latest_free: CachePadded<AtomicUsize>,
}

impl<T> Shared<T> {
    fn new() -> Self {
        Shared {
            mem: [
                UnsafeCell::new(None),
                UnsafeCell::new(None),
                UnsafeCell::new(None),
            ],
            latest_free: CachePadded::new(0.into()),
        }
    }
}

pub fn triple_buffer<T>() -> (Writer<T>, Reader<T>) {
    let chan = Arc::new(Shared::new());

    let w = Writer::new(chan.clone());
    let r = Reader::new(chan);
    (w, r)
}

#[derive(Debug)]
pub struct Reader<T> {
    shared: Arc<Shared<T>>,
    read_idx: usize,
}
unsafe impl<T: Send> Send for Reader<T> {}
unsafe impl<T: Send> Sync for Reader<T> {}

impl<T> Reader<T> {
    fn new(raw_mem: Arc<Shared<T>>) -> Self {
        Reader {
            shared: raw_mem,
            read_idx: 1,
        }
    }

    #[inline]
    pub fn try_read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let has_new_data = self.shared.latest_free.load(Ordering::Acquire) & NEW_DATA_FLAG > 0;
        if has_new_data {
            self.read_idx = self
                .shared
                .latest_free
                .swap(self.read_idx, Ordering::AcqRel)
                & INDEX_MASK;
        }

        #[cfg(loom)]
        let val = unsafe { self.shared.mem[self.read_idx].get().deref() }.clone();
        #[cfg(not(loom))]
        let val = unsafe { &*self.shared.mem[self.read_idx].get() }.clone();
        val
    }
}

#[derive(Debug)]
pub struct Writer<T> {
    shared: Arc<Shared<T>>,
    write_idx: usize,
    last_written: Option<usize>,
}
unsafe impl<T: Send> Send for Writer<T> {}
unsafe impl<T: Send> Sync for Writer<T> {}

impl<T> Writer<T> {
    fn new(raw_mem: Arc<Shared<T>>) -> Self {
        Writer {
            shared: raw_mem,
            write_idx: 2,
            last_written: None,
        }
    }

    #[inline]
    pub fn try_read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let last_written = self.last_written?;

        #[cfg(loom)]
        let val = unsafe { self.shared.mem[last_written].get().deref() }.clone();
        #[cfg(not(loom))]
        let val = unsafe { &*self.shared.mem[last_written].get() }.clone();
        val
    }

    #[inline]
    pub fn write(&mut self, data: T) {
        #[cfg(loom)]
        unsafe {
            self.shared.mem[self.write_idx & INDEX_MASK]
                .get_mut()
                .with(|ptr| {
                    let _ = ptr.replace(Some(data));
                });
        }
        #[cfg(not(loom))]
        // Drop old value and write new one
        let _ = unsafe {
            self.shared.mem[self.write_idx & INDEX_MASK]
                .get()
                .replace(Some(data))
        };

        // Store index
        self.last_written = Some(self.write_idx & INDEX_MASK);
        self.write_idx = self
            .shared
            .latest_free
            .swap(self.write_idx | NEW_DATA_FLAG, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let (mut w, mut r) = triple_buffer();
        w.write(vec![0; 15]);

        assert_eq!(w.try_read(), Some(vec![0; 15]));
        assert_eq!(r.try_read(), Some(vec![0; 15]));
    }

    #[test]
    fn test_read_none() {
        let (mut w, mut r) = triple_buffer();
        assert_eq!(r.try_read(), None);
        w.write(vec![0; 15]);
        assert_eq!(r.try_read(), Some(vec![0; 15]));
    }
}
