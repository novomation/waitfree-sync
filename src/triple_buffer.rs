use std::mem;

use crate::import::{Arc, AtomicUsize, Ordering, UnsafeCell};
use crate::inital_write_guard::InitialWriteGuard;
use crossbeam_utils::CachePadded;

const NEW_DATA_FLAG: usize = 0b100;
const INDEX_MASK: usize = 0b011;

#[derive(Debug)]
struct Slot<T: Sized> {
    mem: [UnsafeCell<T>; 3],
    latest_free: CachePadded<AtomicUsize>,
    write_guard: InitialWriteGuard,
}

impl<T> Slot<T> {
    fn new() -> Self {
        Slot {
            mem: [
                UnsafeCell::new(unsafe { mem::zeroed() }),
                UnsafeCell::new(unsafe { mem::zeroed() }),
                UnsafeCell::new(unsafe { mem::zeroed() }),
            ],
            latest_free: CachePadded::new(0.into()),
            write_guard: InitialWriteGuard::new(),
        }
    }
}

pub fn triple_buffer<T>() -> (Reader<T>, Writer<T>) {
    let chan = Arc::new(TripleBufferRaw::new());

    let r = Reader::new(chan.clone());
    let w = Writer::new(chan);
    (r, w)
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

    pub fn read(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let slot = unsafe { &*self.raw_mem.slot };
        if !slot.write_guard.has_data() {
            return None;
        }
        let has_new_data = slot.latest_free.load(Ordering::Acquire) & NEW_DATA_FLAG > 0;
        if has_new_data {
            self.read_idx = slot.latest_free.swap(self.read_idx, Ordering::AcqRel) & INDEX_MASK;
        }

        #[cfg(loom)]
        let val = unsafe { slot.mem[self.read_idx].get().deref().clone() };
        #[cfg(not(loom))]
        let val = unsafe { slot.mem[self.read_idx].get().read().clone() };
        Some(val)
    }
}

#[derive(Debug)]
pub struct Writer<T> {
    raw_mem: Arc<TripleBufferRaw<T>>,
    write_idx: usize,
}
unsafe impl<T: Send> Send for Writer<T> {}
unsafe impl<T: Send> Sync for Writer<T> {}

impl<T> Writer<T> {
    fn new(raw_mem: Arc<TripleBufferRaw<T>>) -> Self {
        Writer {
            raw_mem,
            write_idx: 2,
        }
    }
    pub fn write(&mut self, data: T) {
        let slot = unsafe { &*self.raw_mem.slot };

        #[cfg(loom)]
        unsafe {
            *slot.mem[self.write_idx & INDEX_MASK].get_mut().deref() = data
        };
        #[cfg(not(loom))]
        unsafe {
            slot.mem[self.write_idx & INDEX_MASK].get().write(data);
        };
        self.write_idx = slot
            .latest_free
            .swap(self.write_idx | NEW_DATA_FLAG, Ordering::AcqRel);
        slot.write_guard.set_has_data();
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
