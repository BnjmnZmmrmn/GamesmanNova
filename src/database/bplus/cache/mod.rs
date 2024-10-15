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

use error::CacheError;
use page::Page;
use super::file::manager::FileManager;

/* 3P IMPORTS */

use anyhow::Result;

/* SUB MODULES */

pub mod error; // error utility
mod manager;   // cache manager (cache api)
mod page;      // page for memory abstraction

/* USEFUL TYPES */

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
    valid: bool,    // indicates if id - page mapping is accurate, or garbage
    id: PageId,     // assigned PageId
    page: Page<'a>, // page for reading and writing
}

struct Cache<'a> {
    policy: EvictionPolicy,               // policy in use by cache
    last_evict: usize,                    // idx of last evicted entry
    capacity: usize,                      // max number of entries allowed in cache
    file_manager: Box<FileManager<'a>>,   // file manager for fetching and flushing pages
    entries: Vec<RwLock<CacheEntry<'a>>>, // list of locked cache entries
    max_fetch_attempts: usize,            // max fetch attempts before throwing error
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

    /// Creates a new cache with `capacity` entries, using `policy` to determine evictions.
    /// 
    /// When fetching entries, the cache will try `max_fetch_attempts` to aquire an entry before
    /// throwing an error. 
    /// 
    /// `file_manager` provides a [`FileManager`] to handle necessary fetching and flushing
    /// of pages to disk.
    fn new(capacity: usize, policy: EvictionPolicy, max_fetch_attempts: usize, file_manager: Box<FileManager<'a>>) -> Cache<'a> {
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

    // returns a 
    fn fetch_entry (
        &self,
        id: PageId,
    ) -> Result<Box<RwLockReadGuard<CacheEntry<'a>>>, CacheError> {
        for _ in 0..self.max_fetch_attempts {
            match self.lookup(id) {
                Ok(idx) => {
                    match self.entries.get(idx) {
                        Some(locked_entry) => {
                            let guard: RwLockReadGuard<CacheEntry<'a>> = locked_entry.read()?;
                            if guard.get_id() == id {
                                return Ok(Box::new(guard));
                            }
                        },
                        _ => continue,
                    }
                }
                Err(_) => self.evict_and_replace(id)?,
            }
        }
        Err(CacheError::FetchFailure(id, self.max_fetch_attempts))
    }

    fn fetch_mut_entry (
        &self,
        id: PageId,
    ) -> Result<Box<RwLockWriteGuard<CacheEntry<'a>>>, CacheError> {
        for _ in 0..self.max_fetch_attempts {
            match self.lookup(id) {
                Ok(idx) => {
                    match self.entries.get(idx) {
                        Some(locked_entry) => {
                            let guard: RwLockWriteGuard<CacheEntry<'a>> = locked_entry.write()?;
                            if guard.get_id() == id {
                                return Ok(Box::new(guard));
                            }
                        },
                        _ => continue,
                    }
                }
                Err(_) => self.evict_and_replace(id)?,
            }
        }
        Err(CacheError::FetchFailure(id, self.max_fetch_attempts))
    }

    fn lookup(&self, id: PageId) -> Result<usize, CacheError> {
        for idx in 0..self.capacity {
            match self.entries.get(idx) {
                Some(locked_entry) => {
                    let read_guard = locked_entry.read()?;
                    if (*read_guard).get_id() == id {
                        return Ok(idx)
                    }
                }
                _ => continue,
            }
        }
        Err(CacheError::LookupFailure(id))
    }

    fn evict_and_replace(&self, id: PageId) -> Result<(), CacheError>{
        match self.policy {
            EvictionPolicy::FIFO => {
                self.last_evict += 1;
                if self.last_evict == self.capacity {
                    self.last_evict = 0;
                }
                match self.entries.get(self.last_evict) {
                    Some(locked_entry) => {
                        let entry: CacheEntry<'a> = *(locked_entry.write()?);
                        entry.id = id;
                        let data = self.file_manager.fetch_page_data_from_disk(id);
                        entry.page.write_at(0, data)?;
                        Ok(())
                    },
                }
            }
            EvictionPolicy::LFU => {
                todo!()
            }
            EvictionPolicy::LRU => {
                todo!()
            }
            EvictionPolicy::MRU => {
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}