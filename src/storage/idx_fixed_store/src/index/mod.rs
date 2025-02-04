use common::ids::{PageId, StateType};

//TODO milestone idx1 - CHANGE THIS TO CHANGE THE INDEX USED
pub const INDEX_TYPE: StateType = StateType::HashTable;

// This must be a power of 2 for extendible hashing to work
pub const STARTING_PAGE_CAPACITY: PageId = 8;

pub mod fixed_index_file;
pub mod fixed_index_page;
pub mod fixed_index_trait;
pub mod fixed_index_tests;
