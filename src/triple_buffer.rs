use crate::import::{Arc, AtomicUsize, Ordering, UnsafeCell};
use crossbeam_utils::CachePadded;

const NEW_DATA_FLAG: usize = 0b100;
const INDEX_MASK: usize = 0b011;

#[derive(Debug)]
struct Slot<T: Sized> {
    mem: [UnsafeCell<Option<T>>; 3],
    latest_free: CachePadded<AtomicUsize>,
}

impl<T> Slot<T> {
    fn new() -> Self {
        Slot {
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
    let chan = Arc::new(TripleBufferRaw::new());

    let w = Writer::new(chan.clone());
    let r = Reader::new(chan);
    (w, r)
}

#[derive(Debug)]
pub struct Reader<T> {
    raw_mem: Arc<TripleBufferRaw<T>>,
    read_idx: usize,
}
unsafe impl<T: Send> Send for Reader<T> {}
unsafe impl<T: Send> Sync for Reader<T> {}

impl<T> Reader<T> {
    fn new(raw_mem: Arc<TripleBufferRaw<T>>) -> Self {
        Reader {
            raw_mem,
            read_idx: 1,
        }
    }

    #[inline]
    pub fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let slot = unsafe { &*self.raw_mem.slot };
        let has_new_data = slot.latest_free.load(Ordering::Acquire) & NEW_DATA_FLAG > 0;
        if has_new_data {
            self.read_idx = slot.latest_free.swap(self.read_idx, Ordering::AcqRel) & INDEX_MASK;
        }

        #[cfg(loom)]
        let val = unsafe { slot.mem[self.read_idx].get().deref() }.clone();
        #[cfg(not(loom))]
        let val = unsafe { &*slot.mem[self.read_idx].get() }.clone();
        val
    }
}

#[derive(Debug)]
pub struct Writer<T> {
    raw_mem: Arc<TripleBufferRaw<T>>,
    write_idx: usize,
    last_written: Option<usize>,
}
unsafe impl<T: Send> Send for Writer<T> {}
unsafe impl<T: Send> Sync for Writer<T> {}

impl<T> Writer<T> {
    fn new(raw_mem: Arc<TripleBufferRaw<T>>) -> Self {
        Writer {
            raw_mem,
            write_idx: 2,
            last_written: None,
        }
    }

    #[inline]
    pub fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let last_written = self.last_written?;
        let slot = unsafe { &*self.raw_mem.slot };

        #[cfg(loom)]
        let val = unsafe { slot.mem[last_written].get().deref() }.clone();
        #[cfg(not(loom))]
        let val = unsafe { &*slot.mem[last_written].get() }.clone();
        val
    }

    #[inline]
    pub fn write(&mut self, data: T) {
        let slot = unsafe { &*self.raw_mem.slot };

        #[cfg(loom)]
        unsafe {
            let _ = (*slot.mem[self.write_idx & INDEX_MASK].get_mut().deref()).insert(data);
        };
        #[cfg(not(loom))]
        unsafe {
            // Drop old value and write new one
            let _ = (*slot.mem[self.write_idx & INDEX_MASK].get()).insert(data);
        };
        // Store index
        self.last_written = Some(self.write_idx & INDEX_MASK);
        self.write_idx = slot
            .latest_free
            .swap(self.write_idx | NEW_DATA_FLAG, Ordering::AcqRel);
    }
}

/// Wrapper around a raw pointer `slot` to a [Slot] to enable manually memory management
#[derive(Debug)]
struct TripleBufferRaw<T: Sized> {
    /// Raw pointer to a [Slot]
    slot: *mut Slot<T>,
}

impl<T> TripleBufferRaw<T> {
    fn new() -> TripleBufferRaw<T> {
        let mem = Box::new(Slot::new());
        // We leak the Slot here to manage the memory our self.
        // The leaked memory is dropped in the Drop impl
        let ptr = Box::leak(mem);
        TripleBufferRaw { slot: ptr }
    }
}

impl<T> Drop for TripleBufferRaw<T> {
    fn drop(&mut self) {
        // This is required to drop the memory allocated in [TripleBufferRaw<T>::new()]
        unsafe { drop(Box::from_raw(self.slot)) };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(loom)]
    use loom::thread;
    #[cfg(not(loom))]
    use std::thread;

    #[cfg(loom)]
    const COUNT: i32 = 4;
    #[cfg(all(not(loom), not(miri)))]
    const COUNT: i32 = 10_000;
    #[cfg(miri)]
    const COUNT: i32 = 1000;

    #[test]
    fn smoke() {
        let (mut w, mut r) = triple_buffer();
        w.write(vec![0; 15]);
        assert_eq!(w.read(), Some(vec![0; 15]));
        assert_eq!(r.read(), Some(vec![0; 15]));
    }

    #[cfg(not(loom))]
    #[test]
    fn test_read_on_writer_multithreaded() {
        let (mut w, mut r) = triple_buffer();
        w.write(vec![0; 15]);
        assert_eq!(r.read(), Some(vec![0; 15]));
        let writer_thread = thread::spawn(move || {
            thread::park();
            for i in 0..COUNT {
                w.write(vec![i; 15]);
                assert_eq!(w.read(), Some(vec![i; 15]));
            }
        });
        let reader_thread = thread::spawn(move || {
            thread::park();
            for _ in 0..COUNT {
                if let Some(val) = r.read() {
                    let first_entry = val[0];
                    for entry in val {
                        assert_eq!(entry, first_entry);
                    }
                }
            }
        });
        writer_thread.thread().unpark();
        reader_thread.thread().unpark();
        assert!(writer_thread.join().is_ok());
        assert!(reader_thread.join().is_ok());
    }
}
