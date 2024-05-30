use core::slice;
use std::io::{Cursor, Read};

const PAGE_SIZE: u16 = 4096;

pub enum PageType {
    ROOT,
    INTERNAL,
    LEAF,
}

struct PageHeader {
    id: u32,
    page_type: PageType,
    num_tuples: u16,
    deleted_tuples: u16,
    // flags: u8,
}

const BUFFER_SIZE: usize = (PAGE_SIZE as usize) - std::mem::size_of::<PageHeader>();

pub struct SlottedPage {
    header: PageHeader,
    buffer: [u8; BUFFER_SIZE],
}

struct TupleMetadata {
    /// Offset into the slotted page buffer
    offset: u16,
    /// size of the tuple
    length: u16,
}

impl TupleMetadata {
    // TODO: switch to Result and maybe use traansmute
    fn from_slice(buffer: &[u8]) -> Self {
        assert_eq!(buffer.len(), 4);

        let mut cursor = Cursor::new(buffer);

        let mut offset_bytes = [0; 2];
        cursor.read_exact(&mut offset_bytes).unwrap();

        let mut length_bytes = [0; 2];
        cursor.read_exact(&mut length_bytes).unwrap();

        let offset = u16::from_le_bytes(offset_bytes); // Assuming little endian
        let length = u16::from_le_bytes(length_bytes); // Assuming little endian

        TupleMetadata { offset, length }
    }
}

impl SlottedPage {
    ///
    /// Constructs a new empty page with the given id and type
    ///
    pub fn new(id: u32, page_type: PageType) -> Self {
        let header: PageHeader = PageHeader {
            id,
            page_type,
            num_tuples: 0,
            deleted_tuples: 0,
        };
        SlottedPage {
            header,
            buffer: [0; BUFFER_SIZE],
        }
    }

    fn get_new_tuple_offset<T>(&self) -> u16 {
        let last_tuple_metadata = match self.header.num_tuples {
            0 => TupleMetadata {
                offset: (BUFFER_SIZE as u16) - 1,
                length: 0,
            },
            tuple_index => {
                let start = (tuple_index as usize) * std::mem::size_of::<TupleMetadata>();
                let end = start + std::mem::size_of::<TupleMetadata>();
                TupleMetadata::from_slice(&self.buffer[start..end])
            }
        };

        last_tuple_metadata.offset - (std::mem::size_of::<T>() as u16)
    }

    // TODO: clean the fuck up
    fn insert<T>(&mut self, tuple: T) {
        let tuple_size = std::mem::size_of::<T>();
        let metadata_size = std::mem::size_of::<TupleMetadata>();

        let tuple_offset = self.get_new_tuple_offset::<T>() as usize;
        let metadata_offset =
            (self.header.num_tuples as usize) * std::mem::size_of::<TupleMetadata>();

        let raw_ptr: *const T = &tuple;

        let tuple_metadata = TupleMetadata {
            offset: tuple_offset as u16,
            length: tuple_size as u16,
        };

        let raw_meta_ptr: *const TupleMetadata = &tuple_metadata;

        let byte_slice: &[u8] =
            unsafe { std::slice::from_raw_parts(raw_ptr as *const u8, tuple_size) };

        let metadata_slice: &[u8] =
            unsafe { std::slice::from_raw_parts(raw_meta_ptr as *const u8, metadata_size) };

        let tuple_copy_range = tuple_offset..(tuple_offset + tuple_size);

        let meta_copy_range = metadata_offset..(metadata_offset + metadata_size);

        // Copy data from source slice to target array
        self.buffer[tuple_copy_range].copy_from_slice(byte_slice);

        self.buffer[meta_copy_range].copy_from_slice(metadata_slice);

        self.header.num_tuples += 1;
    }
}
