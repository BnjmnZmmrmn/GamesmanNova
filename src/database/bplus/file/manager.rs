use crate::database::bplus::cache::error::PageError;
use crate::database::bplus::cache::{Byte, PageId};
use std::marker::PhantomData;

pub(in crate::database::bplus) struct FileManager<'a> {
    phantom: PhantomData<&'a usize>,
}

impl<'a> FileManager<'a> {
    pub(in crate::database::bplus) fn new() -> FileManager<'a> {
        FileManager {
            phantom: PhantomData,
        }
    }

    pub(in crate::database::bplus) fn read_page_at(
        &mut self,
        id: PageId,
        seek: usize,
        length: usize,
    ) -> Result<Vec<Byte>, PageError> {
        todo!()
    }

    pub(in crate::database::bplus) fn write_page_at(
        &mut self,
        id: PageId,
        seek: usize,
        data: Vec<Byte>,
    ) -> Result<(), PageError> {
        todo!()
    }

    pub(in crate::database::bplus) fn fetch_page_data_from_disk(
        &mut self,
        id: PageId,
    ) -> Vec<Byte> {
        todo!()
    }
}
