//! Poison-tolerant lock helpers.
//!
//! `RwLock`/`Mutex` in std return `PoisonError` whenever a thread panics
//! while holding the lock.  The conventional `.unwrap()` then propagates
//! the panic to every subsequent reader/writer — a single panic in one
//! thread cascade-crashes every other thread that touches the lock.
//!
//! For exhale we want different semantics:
//!   * The controller thread, the per-overlay render threads, the
//!     settings-manager flusher, and the main thread *all* touch
//!     `Arc<RwLock<Settings>>`.
//!   * A panic inside (say) egui's settings UI tree shouldn't take down
//!     the breathing animation.
//!   * The poisoned value is almost always still valid — `Settings` is
//!     plain data, no in-flight invariants to worry about.
//!
//! These helpers take the inner value either way: on `Ok` the normal
//! guard, on `Err` the wrapped guard via `PoisonError::into_inner`.  A
//! warning is logged the first time a given lock is observed poisoned
//! so it's not silently swallowed
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait RwLockPoisonExt<T> {
    /// Read-acquire; on poison, log once and continue with the wrapped guard.
    fn read_or_recover(&self) -> RwLockReadGuard<'_, T>;
    /// Write-acquire; on poison, log once and continue with the wrapped guard.
    fn write_or_recover(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockPoisonExt<T> for RwLock<T> {
    fn read_or_recover(&self) -> RwLockReadGuard<'_, T> {
        match self.read() {
            Ok(g)  => g,
            Err(p) => {
                log::warn!("RwLock poisoned (read) — recovering with the wrapped value");
                p.into_inner()
            }
        }
    }
    fn write_or_recover(&self) -> RwLockWriteGuard<'_, T> {
        match self.write() {
            Ok(g)  => g,
            Err(p) => {
                log::warn!("RwLock poisoned (write) — recovering with the wrapped value");
                p.into_inner()
            }
        }
    }
}

pub trait MutexPoisonExt<T> {
    /// Lock; on poison, log once and continue with the wrapped guard.
    fn lock_or_recover(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexPoisonExt<T> for Mutex<T> {
    fn lock_or_recover(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(g)  => g,
            Err(p) => {
                log::warn!("Mutex poisoned — recovering with the wrapped value");
                p.into_inner()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn rwlock_survives_poisoned_writer() {
        let lock = Arc::new(RwLock::new(7_i32));
        let l2 = Arc::clone(&lock);
        let _ = thread::spawn(move || {
            let _g = l2.write().unwrap();
            panic!("intentional poison");
        }).join();
        // Now the lock is poisoned.  `.unwrap()` would panic; ours recovers.
        let v = *lock.read_or_recover();
        assert_eq!(v, 7);
        *lock.write_or_recover() = 9;
        assert_eq!(*lock.read_or_recover(), 9);
    }

    #[test]
    fn mutex_survives_poisoned_holder() {
        let lock = Arc::new(Mutex::new(String::from("hello")));
        let l2 = Arc::clone(&lock);
        let _ = thread::spawn(move || {
            let _g = l2.lock().unwrap();
            panic!("intentional poison");
        }).join();
        let g = lock.lock_or_recover();
        assert_eq!(&*g, "hello");
    }

    #[test]
    fn unpoisoned_lock_works_normally() {
        let lock = RwLock::new(42);
        assert_eq!(*lock.read_or_recover(), 42);
        *lock.write_or_recover() = 100;
        assert_eq!(*lock.read_or_recover(), 100);
    }
}
