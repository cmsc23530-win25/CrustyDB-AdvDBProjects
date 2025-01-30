use crate::buffer_pool::BufferPoolTrait;
use crate::index::fixed_index_trait::IndexFileTrait;
use crate::prelude::*;
use common::prelude::*;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use txn_manager::lockmanager::LockManager;

pub struct FixedIndexFile<T: BufferPoolTrait> {
    bp: Arc<T>,
    lm: Arc<LockManager>,
    c_id: ContainerId,
    supports_range: bool,
    // TODO idx1 Add more fields here as needed
}

impl<T: BufferPoolTrait> IndexFileTrait<T> for FixedIndexFile<T> {
    fn new(
        c_id: ContainerId,
        bp: Arc<T>,
        lm: Arc<LockManager>,
        supports_range: bool,
        initial_page_capacity: PageId,
    ) -> Self {
        panic!("TODO milestone idx1");
    }

    fn add(
        &self,
        search_key: &[u8; SEARCH_KEY_SIZE],
        pointer: &[u8; INDEX_POINTER_SIZE],
        txn: &TransactionId,
    ) -> Result<ValueId, CrustyError> {
        panic!("TODO milestone idx1");
    }

    fn get_pointers_for_key(
        &self,
        search_key: &[u8; SEARCH_KEY_SIZE],
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError> {
        panic!("TODO milestone idx1");
    }

    fn get_pointers_for_key_range(
        &self,
        search_key_min_inclusive: &[u8; SEARCH_KEY_SIZE],
        search_key_max_exclusive: &[u8; SEARCH_KEY_SIZE],
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError> {
        panic!("TODO milestone idx1");
    }

    fn bulk_add(
        &self,
        search_keys: Vec<&[u8; SEARCH_KEY_SIZE]>,
        pointers: Vec<[u8; INDEX_POINTER_SIZE]>,
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError> {
        panic!("TODO milestone idx1");
    }

    fn update_key(
        &self,
        old_search_key: &[u8; SEARCH_KEY_SIZE],
        new_search_key: &[u8; SEARCH_KEY_SIZE],
        pointer: &[u8; INDEX_POINTER_SIZE],
        txn: &TransactionId,
    ) -> Result<ValueId, CrustyError> {
        panic!("TODO milestone idx1");
    }

    fn delete_entry(
        &self,
        search_key: &[u8; SEARCH_KEY_SIZE],
        pointer: &[u8; INDEX_POINTER_SIZE],
        txn: &TransactionId,
    ) -> Result<ValueId, CrustyError> {
        panic!("TODO milestone idx1");
    }

    fn get_pages_used(&self) -> usize {
        panic!("TODO milestone idx1");
    }

}
