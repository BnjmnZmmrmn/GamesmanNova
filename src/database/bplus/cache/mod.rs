//! Cache
//!
//! This module contains a cache implementation that stores
//! pages in cache entries, with additional metadata for
//! reading, writing, fetching, and flushing pages. Also
//! included is an error module for common errors, a cache
//! manager module for high-level cache access, and a page
//! module for a memory abstraction.

/* STANDARD IMPORTS */

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/* CRATE IMPORTS */

use super::file::manager::FileManager;
use error::CacheError;
use page::Page;

/* 3P IMPORTS */

use anyhow::Result;

/* SUB MODULES */

pub mod error; // error utility
mod manager; // cache manager (cache api)
mod page; // page for memory abstraction

/* USEFUL TYPES AND CONSTANTS */

const PAGE_SIZE: usize = 4096;
pub type Byte = u8;
pub type PageId = usize;

/* DEFINITIONS */

/// Enumerates all supported cache eviction polcies.
enum EvictionPolicy {
    /// First in, first out
    FIFO,

    /// Least frequently used
    LFU,

    /// Least recently used
    LRU,

    /// Most recently used
    MRU,
}

struct CacheEntry<'a> {
    valid: bool, // indicates if id - page mapping is accurate, or garbage
    id: PageId,  // assigned PageId
    page: Page<'a>, // page for reading and writing
}

struct Cache<'a> {
    policy: EvictionPolicy, // policy in use by cache
    last_evict: usize,      // idx of last evicted entry
    capacity: usize,        // max number of entries allowed in cache
    file_manager: Box<FileManager<'a>>, // file manager for fetching and flushing pages
    entries: Vec<RwLock<CacheEntry<'a>>>, // list of locked cache entries
    max_fetch_attempts: usize, // max fetch attempts before throwing error
}

/* IMPLEMENTATIONS */

impl<'a> CacheEntry<'a> {
    /// Creates a new cache entry by allocating a new page.
    ///
    /// This method is used within [`Cache`] to initilize its list of
    /// locked entries.
    fn new() -> CacheEntry<'a> {
        CacheEntry {
            valid: false,
            id: 0,
            page: Page::allocate(),
        }
    }

    /// Checks if the entry passed as `self` is valid.
    fn is_valid(&self) -> bool {
        self.valid
    }

    // Returns the PageId of the entry passed as `self`.
    fn get_id(&self) -> PageId {
        self.id
    }
}

impl<'a> Cache<'a> {
    /// Creates a new cache with `capacity` [`CacheEntry`]'s, using `policy` to determine evictions.
    ///
    /// When fetching entries, the cache will try `max_fetch_attempts` to aquire an entry before
    /// throwing an error.
    ///
    /// `file_manager` provides a [`FileManager`] to handle necessary fetching and flushing
    /// of pages to disk.
    fn new(
        capacity: usize,
        policy: EvictionPolicy,
        max_fetch_attempts: usize,
        file_manager: Box<FileManager<'a>>,
    ) -> Cache<'a> {
        let mut entries = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            entries.push(RwLock::new(CacheEntry::new())) // lock up cache entries
        }
        Cache {
            policy,
            last_evict: usize::MAX, // set to max so that first FIFO evict overflows to 0
            capacity,
            file_manager,
            entries,
            max_fetch_attempts,
        }
    }

    /// Returns a heap reference to a [`CacheEntry`] guarded with a RwLockReadGuard.
    /// Identical structure to `Cache::fetch_mut_entry()`.
    ///
    /// # Examples
    /// ```
    /// let cache = Cache::new(10, EvictionPolicy::FIFO, 64, FileManager::new());
    /// { // start new scope
    ///     let guard: Box<RwLockReadGuard<CacheEntry<'a>> = cache.fetch_entry(0)?; // fetch page with id of 0
    ///     let entry: CacheEntry<'a> = **guard; // deref to get entry
    ///     let page_data: Vec<Byte> = entry.page.read_at(0, PAGE_SIZE);
    /// } // end scope to unlock
    /// ```
    ///
    ///  # Errors
    /// This function will error if the cache fails to find a valid entry for `id`
    /// after `self.max_fetch_attempts` lookups.
    fn fetch_entry(
        &mut self,
        id: PageId,
    ) -> Result<Box<RwLockReadGuard<CacheEntry<'a>>>, CacheError> {
        for _ in 0..self.max_fetch_attempts {
            match self.lookup(id) {
                Ok(idx) => match self.entries.get(idx) {
                    Some(locked_entry) => {
                        let guard: RwLockReadGuard<CacheEntry<'a>> =
                            locked_entry.read()?; // create read guard
                        if guard.is_valid() && guard.get_id() == id {
                            return Ok(Box::new(guard));
                        }
                    },
                    _ => continue,
                },
                Err(_) => self.evict_and_replace(id)?, // fetch from disk if no entry found
            }
        }
        Err(CacheError::FetchFailure(
            id,
            self.max_fetch_attempts,
        ))
    }

    /// Returns a heap reference to a [`CacheEntry`] guarded with a RwLockWriteGuard.
    /// Identical structure to `Cache::fetch_entry()`.
    ///
    /// # Examples
    /// ```
    /// let cache = Cache::new(10, EvictionPolicy::FIFO, 64, FileManager::new());
    /// { // start new scope
    ///     let guard: Box<RwLockReadGuard<CacheEntry<'a>> = cache.fetch_mut_entry(0)?; // fetch page with id of 0
    ///     let entry: CacheEntry<'a> = **guard; // deref to get entry
    ///     let page_data: Vec<Byte> = vec![1; PAGE_SIZE];
    ///     entry.page.write_at(0, page_data);
    /// } // end scope to unlock
    /// ```
    ///
    ///  # Errors
    /// This function will error if the cache fails to find a valid entry for `id`
    /// after `self.max_fetch_attempts` lookups.
    fn fetch_mut_entry(
        &mut self,
        id: PageId,
    ) -> Result<Box<RwLockWriteGuard<'a, CacheEntry<'a>>>, CacheError> {
        for _ in 0..self.max_fetch_attempts {
            // Attempt to look up the entry first
            let lookup_result = lookup(self.entries, id);

            match lookup_result {
                Ok(idx) => {
                    // Look up the entry in a separate scope
                    if let Some(locked_entry) = self.entries.get_mut(idx) {
                        let guard: RwLockWriteGuard<CacheEntry<'a>> =
                            locked_entry.write()?;
                        if guard.is_valid() && guard.get_id() == id {
                            return Ok(Box::new(guard));
                        }
                    }
                },
                Err(_) => {
                    // No entry found, we can mutate self now
                    self.evict_and_replace(id)?;
                },
            }
        }
        Err(CacheError::FetchFailure(
            id,
            self.max_fetch_attempts,
        ))
    }

    // used to find a valid entry with passed id without acquiring any locks
    fn lookup(
        entries: Vec<RwLock<CacheEntry<'a>>>,
        id: PageId,
    ) -> Result<usize, CacheError> {
        for idx in 0..entries.capacity() {
            match entries.get(idx) {
                Some(locked_entry) => {
                    let read_guard = locked_entry.read()?;
                    if (*read_guard).is_valid() && (*read_guard).get_id() == id
                    {
                        return Ok(idx);
                    }
                },
                _ => continue,
            }
        }
        Err(CacheError::LookupFailure(id)) // throw failure if no valid entry found
    }

    // evicts an entry as determined by policy, flushing if necessary. then fetches new data from disk,
    // placing into new entry
    fn evict_and_replace(&mut self, id: PageId) -> Result<(), CacheError> {
        match self.policy {
            EvictionPolicy::FIFO => {
                self.last_evict = self.last_evict + 1 % self.capacity;
                match self.entries.get(self.last_evict) {
                    Some(locked_entry) => {
                        let mut guard: RwLockWriteGuard<CacheEntry<'a>> =
                            locked_entry.write()?; // acquire exlusive lock
                        if guard.is_valid() && guard.get_id() == id {
                            let data: Vec<Byte> = (*guard)
                                .page
                                .read_at(0, PAGE_SIZE)
                                .map_err(|_OutOfBoundsRead| {
                                    CacheError::FailedCacheRead(id)
                                })?;
                            self.file_manager
                                .flush_page_data_to_disk(
                                    (*guard).get_id(),
                                    data,
                                ); // flush if necessary
                        }
                        (*guard).id = id;
                        let data = self
                            .file_manager
                            .fetch_page_data_from_disk(id); // fetch new data
                        match (*guard).page.write_at(0, data) {
                            Ok(()) => Ok(()),
                            Err(_) => Err(CacheError::FailedCacheWrite(id)), // error conversion
                        }
                    },
                    _ => Err(CacheError::EvictionFailure(self.last_evict)),
                }
            },
            EvictionPolicy::LFU => {
                todo!()
            },
            EvictionPolicy::LRU => {
                todo!()
            },
            EvictionPolicy::MRU => {
                todo!()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
