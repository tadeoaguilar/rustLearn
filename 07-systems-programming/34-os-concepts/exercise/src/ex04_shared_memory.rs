//! Exercise 4: shared memory -- a cache in a memory-mapped file.
//!
//! `mmap` maps a file into the address space. Two processes (or two
//! independent mappings) of the same file with `MAP_SHARED` see the *same
//! physical pages*: a write by one is immediately visible to the other, with
//! no system call in between. That's the fastest IPC there is -- and it
//! makes synchronisation your problem: the kernel won't stop two writers
//! colliding, so the region holds its own lock and atomic counters.
//!
//! Layout (all little-endian, the header 8-byte aligned at the page start):
//!
//! ```text
//! offset 0   u32 magic "RCCH"
//!        4   u32 lock (0 free, 1 held)            -- AtomicU32
//!        8   u64 counter                          -- AtomicU64
//!       16   u32 slot count
//!       20   u32 (padding)
//!       24   slots: [u64 key (0 = empty)][u8 len][55 bytes value]  x slot count
//! ```

use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use memmap2::MmapMut;

pub const MAGIC: u32 = u32::from_le_bytes(*b"RCCH");
pub const HEADER: usize = 24;
pub const SLOT: usize = 64;
pub const MAX_VALUE: usize = SLOT - 9;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("not a cache file")]
    BadMagic,
    #[error("keys are non-zero")]
    ZeroKey,
    #[error("values are at most 55 bytes")]
    ValueTooLong,
    #[error("the cache is full")]
    Full,
}

pub struct SharedCache {
    map: MmapMut,
    slots: usize,
}

/// Holds the region's spin lock; releases it on drop.
pub struct LockGuard<'a> {
    lock: &'a AtomicU32,
}

impl Drop for LockGuard<'_> {
    fn drop(&mut self) {
        self.lock.store(0, Ordering::Release);
    }
}

impl SharedCache {
    /// Create (or truncate) `path` with room for `slots` entries, and map it.
    pub fn create(path: &Path, slots: usize) -> Result<Self, CacheError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.set_len((HEADER + slots * SLOT) as u64)?;
        // SAFETY: we just created and sized the file; other mappings of it are
        // the point (shared memory), and all shared fields are accessed atomically
        // or under the region's lock.
        let mut map = unsafe { MmapMut::map_mut(&file)? };
        map[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        map[16..20].copy_from_slice(&(slots as u32).to_le_bytes());
        Ok(SharedCache { map, slots })
    }

    /// Map an existing cache file (another process's, or another mapping).
    pub fn open(path: &Path) -> Result<Self, CacheError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        // SAFETY: as in `create`.
        let map = unsafe { MmapMut::map_mut(&file)? };
        if map.len() < HEADER || map[0..4] != MAGIC.to_le_bytes() {
            return Err(CacheError::BadMagic);
        }
        let slots = u32::from_le_bytes(map[16..20].try_into().expect("4 bytes")) as usize;
        if map.len() < HEADER + slots * SLOT {
            return Err(CacheError::BadMagic);
        }
        Ok(SharedCache { map, slots })
    }

    fn lock_word(&self) -> &AtomicU32 {
        // SAFETY: the mapping starts page-aligned, so offset 4 is 4-aligned;
        // it lives as long as `self`; every access to it is atomic.
        unsafe { &*(self.map.as_ptr().add(4) as *const AtomicU32) }
    }

    fn counter(&self) -> &AtomicU64 {
        // SAFETY: offset 8 of a page-aligned mapping is 8-aligned; as above.
        unsafe { &*(self.map.as_ptr().add(8) as *const AtomicU64) }
    }

    /// Spin until the lock is ours.
    pub fn lock(&self) -> LockGuard<'_> {
        todo!("Exercise 4")
    }

    /// Atomically add one to the shared counter; returns the new value.
    pub fn increment(&self) -> u64 {
        todo!("Exercise 4")
    }

    pub fn count(&self) -> u64 {
        todo!("Exercise 4")
    }

    /// Raw pointer to byte `offset` of the region. Slot bytes are reached
    /// only through raw pointers, under the lock: another process may write
    /// them at any time, so Rust references into them would be a lie.
    fn at(&self, offset: usize) -> *mut u8 {
        debug_assert!(offset < self.map.len());
        // SAFETY: `offset` is within the mapping (checked by the callers'
        // index arithmetic against `slots`, fixed at create/open).
        unsafe { self.map.as_ptr().add(offset) as *mut u8 }
    }

    fn slot_offset(&self, index: usize) -> usize {
        todo!("Exercise 4")
    }

    fn slot_key(&self, index: usize) -> u64 {
        todo!("Exercise 4")
    }

    /// Insert or replace `key`'s value (linear probing from `key % slots`),
    /// under the region lock -- so it's safe across mappings and processes.
    pub fn put(&self, key: u64, value: &[u8]) -> Result<(), CacheError> {
        todo!("Exercise 4")
    }

    /// The value stored for `key`, if any.
    pub fn get(&self, key: u64) -> Option<Vec<u8>> {
        todo!("Exercise 4")
    }

    /// Push changes to the file (they're already visible to other mappings).
    pub fn flush(&self) -> io::Result<()> {
        todo!("Exercise 4")
    }
}
