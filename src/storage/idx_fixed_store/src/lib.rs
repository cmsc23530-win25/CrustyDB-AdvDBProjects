#[macro_use]
extern crate log;

pub mod buffer_frame;
pub mod buffer_pool;
pub mod fixed_page;
pub mod heap;
pub mod index;
pub mod storage_manager;
pub mod test_util;

pub mod prelude {
    use common::prelude::*;
    pub const STORAGE_DIR: &str = "na";
    pub use common::PAGE_SIZE;

    pub const VALUE_SIZE: usize = PAGE_SIZE / 32;
    pub const KEY_SIZE: usize = VALUE_SIZE / 8;
    pub const SEARCH_KEY_SIZE: usize = 8;
    pub const INDEX_POINTER_SIZE: usize = std::mem::size_of::<common::ids::VidBytes>();
    pub type PagePointer = Option<PageId>;

    pub const INDEX_ENTRY_SIZE: usize = SEARCH_KEY_SIZE + INDEX_POINTER_SIZE;

    pub const DATA_VALUE_COUNT: usize = PAGE_SIZE / (KEY_SIZE + VALUE_SIZE);
    pub const INDEX_VALUE_COUNT: usize = PAGE_SIZE / INDEX_ENTRY_SIZE;

    /// Making fixed size pages easy with limiting slots
    pub const PAGE_SLOT_LIMIT: usize = 1024;

    /// A hash chain should be at most this many pages long
    pub const MAX_CHAIN_LENGTH: usize = 3;

    // A probe should be at most this many buckets long
    pub const MAX_PROBE_LENGTH: usize = 3;

    pub fn extract_search_key(data: &[u8]) -> &[u8; SEARCH_KEY_SIZE] {
        data[VALUE_SIZE - SEARCH_KEY_SIZE..]
            .try_into()
            .expect("slice with incorrect length")
    }
}
