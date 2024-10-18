use super::error::*;
use super::{Byte, PageId};
use super::{Cache, CacheEntry, EvictionPolicy};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

pub(in crate::database::bplus) struct CacheManager<'a> {
    cache: Cache<'a>,
}

impl<'a> CacheManager<'a> {
    pub(in crate::database::bplus) fn new(
        cache_capacity: usize,
        cache_policy: EvictionPolicy,
        max_fetch_attempts: usize,
    ) -> CacheManager<'a> {
        CacheManager {
            cache: Cache::new(cache_capacity, cache_policy, max_fetch_attempts),
        }
    }

    pub(in crate::database::bplus) fn read_page_at(
        &mut self,
        id: PageId,
        seek: usize,
        length: usize,
    ) -> Result<Vec<Byte>, PageError> {
        let guard: Box<RwLockReadGuard<CacheEntry<'a>>> =
            self.cache.fetch_entry(id)?;
        (*guard).page.read_at(seek, length)
    }

    pub(in crate::database::bplus) fn write_page_at(
        &mut self,
        id: PageId,
        seek: usize,
        data: Vec<Byte>,
    ) -> Result<(), PageError> {
        let mut guard: Box<RwLockWriteGuard<CacheEntry<'a>>> =
            self.cache.fetch_mut_entry(id)?;
        (*guard).page.write_at(seek, data)
    }

    pub(in crate::database::bplus::cache) fn fetch_page_data_from_file(
        &mut self,
        id: PageId,
    ) -> Vec<Byte> {
        todo!()
    }
}
