use super::CacheEntry;
use super::{Page, PageId};
use std::fmt::{Display, Error, Formatter};
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, PartialEq)]
pub enum PageError {
    OutOfBoundsRead(String),
    OutOfBoundsWrite(String),
    PageNotFound(PageId),
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum CacheError {
    LookupFailure(PageId),
    FetchFailure(PageId, usize),
    FailedCacheRead(PageId),
    FailedCacheWrite(PageId),
    PoisonedCacheEntry,
    Unknown,
}

impl Display for PageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            PageError::OutOfBoundsRead(ref msg) => write!(
                f,
                "Attempting read outside of page dimensions: {}",
                msg
            ),
            PageError::OutOfBoundsWrite(ref msg) => write!(
                f,
                "Attempting write outside of page dimensions: {}",
                msg
            ),
            PageError::PageNotFound(ref id) => {
                write!(f, "Unable to fetch page: {}", id)
            },
            PageError::Unknown => write!(f, "An unknown page error occurred"),
        }
    }
}

impl Display for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            CacheError::LookupFailure(ref id) => {
                write!(f, "Failed to lookup page: {}", id)
            },
            CacheError::FetchFailure(ref id, ref fetch_attempts) => {
                write!(
                    f,
                    "Failed to fetch page after {} attempts: {}",
                    fetch_attempts, id
                )
            },
            CacheError::FailedCacheRead(ref id) => {
                write!(f, "Failed to read page: {}", id)
            },
            CacheError::FailedCacheWrite(ref id) => {
                write!(f, "Failed to write to page: {}", id)
            },
            CacheError::PoisonedCacheEntry => {
                write!(f, "Cache lock is poisonous")
            },
            CacheError::Unknown => write!(f, "An unknown cache error occurred"),
        }
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'_, CacheEntry<'a>>>> for CacheError {
    fn from(_error: PoisonError<RwLockReadGuard<'_, CacheEntry<'a>>>) -> Self {
        CacheError::PoisonedCacheEntry
    }
}

impl<'a> From<PoisonError<RwLockWriteGuard<'_, CacheEntry<'a>>>>
    for CacheError
{
    fn from(_error: PoisonError<RwLockWriteGuard<'_, CacheEntry<'a>>>) -> Self {
        CacheError::PoisonedCacheEntry
    }
}

impl From<PageError> for CacheError {
    fn from(error: PageError) -> Self {
        match error {
            PageError::OutOfBoundsRead(id) => CacheError::FailedCacheRead(id),
            PageError::OutOfBoundsWrite(id) => CacheError::FailedCacheWrite(id),
            _ => CacheError::Unknown,
        }
    }
}

impl From<CacheError> for PageError {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::LookupFailure(ref id) => PageError::PageNotFound(*id),
            _ => PageError::Unknown,
        }
    }
}

impl std::error::Error for PageError {}

impl std::error::Error for CacheError {}
