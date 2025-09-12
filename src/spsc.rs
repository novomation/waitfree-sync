//! Wait-free single-producer single-consumer (SPSC) queue to send data to another thread.
//! Based on the improved FastForward queue.
//!
//! # Example
//! ```rust
//! use waitfree_sync::spsc;
//!
//! //                            Type ──╮  ╭─ Size
//! let (mut tx, mut rx) = spsc::spsc::<u64,8>();
//! tx.write(234);
//! assert_eq!(rx.read(),Some(234u64));
//! ```
//!
//! # Behaviour for full and empty queue.
//! If the queue is full the [Writer] returns an [NoSpaceLeftError]
//! If the queue is empty the [Reader] returns `None`
//!
//! # Behaviuor on drop
//!
use crate::{
    import::{Arc, AtomicBool, Ordering, UnsafeCell},
    // EnqeueueError, ReadPrimitive, WritePrimitive,
};
use core::error::Error;
use crossbeam_utils::CachePadded;
use std::fmt::Debug;

/// Create a new wait-free SPSC queue. The size must be a power of two and is validate during compile time.
/// Therefore you have to provide the size as const generic.
///
/// # Example
/// ```rust
/// use waitfree_sync::spsc;
///
/// //                    Type ──╮  ╭─ Size
/// let (tx, rx) = spsc::spsc::<u64,8>();
/// ```
pub fn spsc<T, const SIZE: usize>() -> (Writer<T>, Reader<T>) {
    const {
        if !is_power_of_two(SIZE) {
            panic!("The SIZE must be a power of 2")
        }
    };

    let chan = Arc::new(SpscRaw::new(SIZE));

    let r = Reader::new(chan.clone());
    let w = Writer::new(chan);

    (w, r)
}

const fn is_power_of_two(x: usize) -> bool {
    let c = x.wrapping_sub(1);
    (x != 0) && (x != 1) && ((x & c) == 0)
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoSpaceLeftError<T>(T);
impl<T: Debug> Error for NoSpaceLeftError<T> {}
impl<T> core::fmt::Display for NoSpaceLeftError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No space left in the spsc queue.")
    }
}

#[derive(Debug)]
struct Slot<T: Sized> {
    value: UnsafeCell<Option<T>>,
    occupied: CachePadded<AtomicBool>,
}
impl<T> Slot<T> {
    fn new() -> Self {
        Self {
            value: UnsafeCell::new(None),
            occupied: CachePadded::new(false.into()),
        }
    }
}

#[derive(Debug)]
struct Spsc<T: Sized> {
    mem: Box<[Slot<T>]>,
    // The mask is written when this structure is created and is then only read.
    // Therefore, we do not need Atomic here.
    mask: usize,
}

impl<T> Spsc<T> {
    fn new(size: usize) -> Self {
        let mut buffer = Vec::with_capacity(size);
        for _ in 0..size {
            buffer.push(Slot::new());
        }
        let buffer: Box<[Slot<T>]> = buffer.into_boxed_slice();
        Spsc {
            mem: buffer,
            mask: size - 1,
        }
    }
    fn size(&self) -> usize {
        self.mask + 1
    }
}

#[derive(Debug)]
pub struct Reader<T> {
    raw_mem: Arc<SpscRaw<T>>,
    read: usize,
}
unsafe impl<T: Send> Send for Reader<T> {}
unsafe impl<T: Send> Sync for Reader<T> {}

impl<T> Reader<T> {
    fn new(raw_mem: Arc<SpscRaw<T>>) -> Self {
        Reader { raw_mem, read: 0 }
    }
}

impl<T> Reader<T> {
    pub fn read(&mut self) -> Option<T> {
        let spsc = unsafe { &*self.raw_mem.spsc };
        let rpos = self.read & spsc.mask;
        let slot = unsafe { spsc.mem.get_unchecked(rpos) };
        if !slot.occupied.load(Ordering::Acquire) {
            None
        } else {
            #[cfg(not(loom))]
            let val = unsafe { slot.value.get().replace(None) };
            #[cfg(loom)]
            let val = unsafe {
                slot.value
                    .get_mut()
                    .with(|ptr| ptr.replace(std::mem::zeroed()))
            };

            slot.occupied.store(false, Ordering::Release);
            self.read += 1;
            val
        }
    }
    pub fn size(&self) -> usize {
        // SAFETY: This is safe because we only read size which is never written.
        let spsc = unsafe { &*self.raw_mem.spsc };
        spsc.size()
    }
}

#[derive(Debug)]
pub struct Writer<T> {
    raw_mem: Arc<SpscRaw<T>>,
    write: usize,
}
unsafe impl<T: Send> Send for Writer<T> {}
unsafe impl<T: Send> Sync for Writer<T> {}
impl<T> Writer<T> {
    fn new(raw_mem: Arc<SpscRaw<T>>) -> Self {
        Writer { raw_mem, write: 0 }
    }
}

impl<T> Writer<T> {
    pub fn write(&mut self, data: T) -> Result<(), NoSpaceLeftError<T>> {
        let spsc = unsafe { &*self.raw_mem.spsc };
        let wpos = self.write & spsc.mask;

        let slot = unsafe { spsc.mem.get_unchecked(wpos) };
        if slot.occupied.load(Ordering::Acquire) {
            Err(NoSpaceLeftError(data))
        } else {
            #[cfg(not(loom))]
            unsafe {
                slot.value.get().write(Some(data))
            };
            #[cfg(loom)]
            unsafe {
                slot.value.get_mut().with(|ptr| ptr.write(Some(data)))
            };
            slot.occupied.store(true, Ordering::Release);
            self.write += 1;
            Ok(())
        }
    }
    pub fn size(&self) -> usize {
        // SAFETY: This is safe because we only read size which is never written.
        let spsc = unsafe { &*self.raw_mem.spsc };
        spsc.size()
    }
}

/// Wrapper around a raw pointer `slot` to a [Slot] to enable manually memory management
#[derive(Debug)]
struct SpscRaw<T: Sized> {
    /// Raw pointer to a [Slot]
    spsc: *mut Spsc<T>,
}

impl<T> SpscRaw<T> {
    fn new(size: usize) -> Self {
        // Allocate a buffer of `cap` slots initialized
        // with stamps.
        let mem = Box::new(Spsc::new(size));
        // We leak the Slot here to manage the memory our self.
        // The leaked memory is dropped in the Drop impl
        let ptr = Box::leak(mem);
        Self { spsc: ptr }
    }
}

impl<T> Drop for SpscRaw<T> {
    fn drop(&mut self) {
        // This is required to drop the memory allocated in [WriteRingBufferRaw<T>::new()]
        unsafe { drop(Box::from_raw(self.spsc)) };
    }
}

#[cfg(not(loom))]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let (mut w, mut r) = spsc::<_, 4>();
        w.write(vec![0; 15]).unwrap();
        w.write(vec![0; 16]).unwrap();
        w.write(vec![0; 17]).unwrap();
        w.write(vec![0; 18]).unwrap();

        assert_eq!(r.read(), Some(vec![0; 15]));
        assert_eq!(r.read(), Some(vec![0; 16]));
        assert_eq!(r.read(), Some(vec![0; 17]));
        assert_eq!(r.read(), Some(vec![0; 18]));
    }

    #[test]
    fn test_is_power_of_two() {
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(!is_power_of_two(3));
        assert!(is_power_of_two(4));
        assert!(!is_power_of_two(5));
        assert!(!is_power_of_two(6));
        assert!(!is_power_of_two(7));
        assert!(is_power_of_two(8));
        assert!(!is_power_of_two(9));

        assert!(!is_power_of_two(15));
        assert!(is_power_of_two(16));
        assert!(!is_power_of_two(17));

        assert!(!is_power_of_two(31));
        assert!(is_power_of_two(32));
        assert!(!is_power_of_two(33));
    }

    #[test]
    fn test_full_empty() {
        let (mut write, mut read) = spsc::<i32, 4>();
        assert_eq!(write.write(1), Ok(()));
        assert_eq!(write.write(2), Ok(()));
        assert_eq!(write.write(3), Ok(()));
        assert_eq!(write.write(4), Ok(()));
        assert_eq!(write.write(5), Err(NoSpaceLeftError(5)));
        assert_eq!(read.read(), Some(1));
        assert_eq!(write.write(6), Ok(()));
        assert_eq!(read.read(), Some(2));
        assert_eq!(read.read(), Some(3));
        assert_eq!(read.read(), Some(4));
        assert_eq!(read.read(), Some(6));
        assert_eq!(read.read(), None);
    }

    #[test]
    fn test_drop_one_side() {
        let (mut write, read) = spsc::<i32, 4>();
        drop(read);
        assert_eq!(write.write(1), Ok(()));
        assert_eq!(write.write(2), Ok(()));
        assert_eq!(write.write(3), Ok(()));
        assert_eq!(write.write(4), Ok(()));
        assert_eq!(write.write(5), Err(NoSpaceLeftError(5)));
    }
}
