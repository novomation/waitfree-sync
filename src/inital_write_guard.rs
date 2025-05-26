use std::sync::atomic::{AtomicBool, Ordering};

/// It provides functions that check whether a UnsafeCell buffer has been initialized.
/// This is like an atomic `Option` as an separate flag.
#[derive(Debug)]
pub struct InitialWriteGuard {
    has_data: AtomicBool,
}

impl InitialWriteGuard {
    /// Create a new [InitialWriteGuard]
    pub fn new() -> Self {
        InitialWriteGuard {
            has_data: false.into(),
        }
    }

    /// True, if data has been added before
    #[inline]
    pub fn has_data(&self) -> bool {
        self.has_data.load(Ordering::Acquire)
    }

    /// Inform the guard, that data has been added
    #[inline]
    pub fn set_has_data(&self) {
        self.has_data.store(true, Ordering::Release);
    }
}

impl Default for InitialWriteGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let wg = InitialWriteGuard::new();
        assert!(!wg.has_data());
        wg.set_has_data();
        assert!(wg.has_data())
    }
}
