//! Page.rs
//! 
//! This module provides a wrapper for heap memory in the
//! form of a page abstraction. It supports operations for
//! allocating a new page, reading from an existing page,
//! and writing to an existing page.


/* STANDARD IMPORTS */

use std::boxed::Box;
use std::marker::PhantomData;

/* CRATE IMPORTS */

use super::Byte;
use super::error::PageError;

/* 3P IMPORTS */

use anyhow::Result;

/* CONSTANTS */

const PAGE_SIZE: usize = 4096;

/* DEFINITIONS */

pub(super) struct Page<'a> {
    data: Box<[Byte; PAGE_SIZE]>,    // PAGE_SIZE bytes of data
    dirty: bool,                     // dirty flag
    phantom: PhantomData<&'a usize>, // for lifetime parameter
}

/* IMPLEMENTATIONS */

impl<'a> Page<'a> {

    /// Creates a new [`Page`] whose data is set to 0^PAGE_SIZE
    /// 
    /// # Examples
    /// ```
    /// let page: Page = Page::allocate();
    /// ```
    pub(super) fn allocate() -> Page<'a> {
        Page {
            data: Box::new([0; PAGE_SIZE]),
            dirty: false,
            phantom: PhantomData,
        }
    }

    /// Reads `length` bytes starting from `seek` from a [`Page`], returning
    /// the data in form `Vec<Byte>`
    /// 
    /// # Examples
    /// ```
    /// let page: Page = Page::allocate();
    /// let some_data: Vec<Byte> = page.read_at(0, 10)?;       // read first 10 bytes of page
    /// let some_more_data: Vec<Byte> = page.read_at(10, 10)?; // read second 10 bytes of page
    /// ```
    /// 
    /// # Errors
    /// 
    /// This function will error if the combination of `seek` and `length`
    /// results in an out of bounds access.
    pub(super) fn read_at(
        &self,
        seek: usize,
        length: usize,
    ) -> Result<Vec<Byte>, PageError> {
        if seek > PAGE_SIZE {
            return Err(PageError::OutOfBoundsRead(format!(
                "Seek {}, Page Size {}",
                seek, PAGE_SIZE
            )));
        }
        if length > PAGE_SIZE {
            return Err(PageError::OutOfBoundsRead(format!(
                "Length {}, Page Size {}",
                length, PAGE_SIZE
            )));
        }
        if seek + length > PAGE_SIZE {
            return Err(PageError::OutOfBoundsRead(format!(
                "Seek + Length {}, Page Size {}",
                seek + length,
                PAGE_SIZE
            )));
        }
        Ok(self.data[seek..seek + length].to_vec())
    }

    /// Writes `data` starting from `seek` to a [`Page`].
    /// 
    /// # Examples
    /// ```
    /// let mut page: Page = Page::allocate();
    /// let some_data: Vec<Byte> = vec![1; 10];
    /// page.write_at(0, some_data)?;  // write to first 10 bytes of page
    /// page.write_at(10, some_data)?; // write to second 10 bytes of page
    /// ```
    /// 
    /// # Errors
    /// 
    /// This function will error if the combination of `seek` and `length`
    /// results in an out of bounds access.
    pub(super) fn write_at(
        &mut self,
        seek: usize,
        data: Vec<Byte>,
    ) -> Result<(), PageError> {
        if seek > PAGE_SIZE {
            return Err(PageError::OutOfBoundsWrite(format!(
                "Seek {}, Page Size {}",
                seek, PAGE_SIZE
            )));
        }
        if data.len() > PAGE_SIZE {
            return Err(PageError::OutOfBoundsWrite(format!(
                "Length {}, Page Size {}",
                data.len(),
                PAGE_SIZE
            )));
        }
        if seek + data.len() > PAGE_SIZE {
            return Err(PageError::OutOfBoundsWrite(format!(
                "Seek + Length {}, Page Size {}",
                seek + data.len(),
                PAGE_SIZE
            )));
        }
        self.data[seek..seek + data.len()].copy_from_slice(&data);
        self.dirty = true;
        Ok(())
    }


    /// Returns if a [`Page`] has been written to.
    /// 
    /// # Examples
    /// ```
    /// let mut page: Page = Page::allocate();
    /// let mut dirty: bool = page.is_dirty();  // dirty will be false
    /// let some_data: Vec<Byte> = vec![1; 10];
    /// page.write_at(0, some_data)?;
    /// dirty = page.is_dirty();                // dirty will be true
    /// ```
    pub(super) fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_out_of_bounds_seek_length_pairs() -> Vec<(usize, usize)> {
        vec![
            (PAGE_SIZE + 1, 0),
            (0, PAGE_SIZE + 1),
            (PAGE_SIZE / 2, PAGE_SIZE / 2 + 1),
        ]
    }

    #[test]
    fn allocate_page() -> Result<()> {
        let page: Page = Page::allocate();
        assert_eq!(page.is_dirty(), false);
        Ok(())
    }

    #[test]
    fn simple_read() -> Result<()> {
        let page: Page = Page::allocate();
        let data: Vec<Byte> = page.read_at(0, PAGE_SIZE)?;
        assert_eq!(data, vec![0; PAGE_SIZE]);
        Ok(())
    }

    #[test]
    fn simple_write() -> Result<()> {
        let mut page: Page = Page::allocate();
        page.write_at(0, vec![1; PAGE_SIZE])?;
        assert_eq!(page.read_at(0, PAGE_SIZE)?, vec![1; PAGE_SIZE]);
        assert_eq!(page.is_dirty(), true);
        Ok(())
    }

    #[test]
    fn read_at_offset() -> Result<()> {
        let page: Page = Page::allocate();
        assert_eq!(
            page.read_at(PAGE_SIZE / 2, PAGE_SIZE / 2)?,
            vec![0; PAGE_SIZE / 2]
        );
        Ok(())
    }

    #[test]
    fn write_at_offset() -> Result<()> {
        let mut page: Page = Page::allocate();
        page.write_at(PAGE_SIZE / 2, vec![1; PAGE_SIZE / 2])?;
        let mut expected: Vec<Byte> = vec![0; PAGE_SIZE / 2];
        expected.extend([1; PAGE_SIZE / 2]);
        assert_eq!(page.read_at(0, PAGE_SIZE)?, expected);
        Ok(())
    }

    #[test]
    fn out_of_bounds_reads() -> Result<()> {
        let page: Page = Page::allocate();
        for (seek, length) in get_out_of_bounds_seek_length_pairs() {
            assert!(matches!(
                page.read_at(seek, length),
                Err(PageError::OutOfBoundsRead(_))
            ));
        }
        Ok(())
    }

    #[test]
    fn out_of_bounds_writes() -> Result<()> {
        let mut page: Page = Page::allocate();
        for (seek, length) in get_out_of_bounds_seek_length_pairs() {
            assert!(matches!(
                page.write_at(seek, vec![1; length]),
                Err(PageError::OutOfBoundsWrite(_))
            ));
        }
        Ok(())
    }
}
