use std::sync::Arc;

use crate::buffer_pool::BufferPoolTrait;
use crate::prelude::*;
use common::prelude::*;
use txn_manager::lockmanager::LockManager;

pub trait IndexFileTrait<T: BufferPoolTrait> {
    /// Create a new index file. This container should have been registered with the buffer pool prior. This
    /// function should allocate the initial pages for the index file with the buffer pool.
    ///
    /// # Arguments
    ///
    /// * `c_id` - The container id of the index file
    /// * `bp` - The buffer pool to use
    /// * `lm` - The lock manager to use
    /// * `supports_range` - Whether the index supports range queries or not
    /// * `initial_page_capacity` - The initial number of pages to allocate for the index. only for hash index
    fn new(
        c_id: ContainerId,
        bp: Arc<T>,
        lm: Arc<LockManager>,
        supports_range: bool,
        initial_page_capacity: PageId,
    ) -> Self;

    /// Add a new entry to the index
    ///
    /// # Arguments
    ///
    /// * `search_key` - The search key for the index entry
    /// * `pointer` - The pointer to the data value id encoded as a 10 byte array
    /// * `txn` - The transaction id for the operation
    ///
    /// # Returns
    ///
    /// * `Ok(ValueID)` The value id of the new entry in the index, note this can move if the index is reorganized
    /// * `Err(CrustyError)` if the entry cannot be added, signaling the transaction should abort.
    fn add(
        &self,
        search_key: &[u8; SEARCH_KEY_SIZE],
        pointer: &[u8; INDEX_POINTER_SIZE],
        txn: &TransactionId,
    ) -> Result<ValueId, CrustyError>;

    /// Add multiple entries to the index in a bulk load
    ///
    /// # Arguments
    ///
    /// * `search_keys` - The vector of search key for the index entries
    /// * `pointers` - The vector of pointers to the data value id encoded as a 10 byte array
    /// * `txn` - The transaction id for the operation
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ValueID>)` The value ids of the new entries in the index, note this can move if the index is reorganized
    /// * `Err(CrustyError)` if the entry cannot be added, signaling the transaction should abort.
    fn bulk_add(
        &self,
        search_keys: Vec<&[u8; SEARCH_KEY_SIZE]>,
        pointers: Vec<[u8; INDEX_POINTER_SIZE]>,
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError>;

    /// Update the key of an existing entry in the index
    ///
    ///
    /// # Arguments
    ///
    /// * `old_search_key` - The prior search key for the index entry
    /// * `new_search_key` - The updated search key for the index entry
    /// * `pointer` - The pointer to the data value id encoded as a 10 byte array. Needed for disambiguating if the old key is not unique
    /// * `txn` - The transaction id for the operation
    ///
    /// # Returns
    ///
    /// * `Ok(ValueID)` The value id of the update entry in the index, note this can move if the index is reorganized
    /// * `Err(CrustyError)` if the entry cannot be added, signaling the transaction should abort.
    fn update_key(
        &self,
        old_search_key: &[u8; SEARCH_KEY_SIZE],
        new_search_key: &[u8; SEARCH_KEY_SIZE],
        pointer: &[u8; INDEX_POINTER_SIZE],
        txn: &TransactionId,
    ) -> Result<ValueId, CrustyError>;

    /// Delete an entry from the index
    ///
    /// # Arguments
    ///
    /// * `search_key` - The search key for the index entry to delete
    /// * `pointer` - The pointer to the data value id encoded as a 10 byte array. Needed for disambiguating if the key is not unique
    /// * `txn` - The transaction id for the operation
    ///
    /// # Returns
    ///
    /// * `Ok(ValueID)` The value id of the new entry in the index, note this can move if the index is reorganized
    /// * `Err(CrustyError)` if the entry cannot be added, signaling the transaction should abort.
    fn delete_entry(
        &self,
        search_key: &[u8; SEARCH_KEY_SIZE],
        pointer: &[u8; INDEX_POINTER_SIZE],
        txn: &TransactionId,
    ) -> Result<ValueId, CrustyError>;

    /// Get the pointers for a specific key in the index (equality search)
    ///
    /// # Arguments
    ///
    /// * `search_key` - The search key for the index entry to delete
    /// * `pointer` - The pointer to the data value id encoded as a 10 byte array. Needed for disambiguating if the key is not unique
    /// * `txn` - The transaction id for the operation
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ValueId>)` The vector of value ids of the entries in the data file, decoded from the byte array via ValueId::from_bytes
    /// * `Err(CrustyError)` if the lookup cannot be performed.
    fn get_pointers_for_key(
        &self,
        search_key: &[u8; SEARCH_KEY_SIZE],
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError>;

    /// Get the pointers for a range of keys in the index
    ///
    /// # Arguments
    ///
    /// * `search_key_min_inclusive` - The minimum inclusive search key for the index to find
    /// * `search_key_max_exclusive` - The max exclusive search key for the index to find
    /// * `txn` - The transaction id for the operation
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ValueId>)` The vector of value ids of the entries in the data file, decoded from the byte array via ValueId::from_bytes
    /// * `Err(CrustyError)` if the lookup cannot be performed. If called on a non-range index, this should return a Err(CrustyError::InvalidOperation).
    fn get_pointers_for_key_range(
        &self,
        search_key_min_inclusive: &[u8; SEARCH_KEY_SIZE],
        search_key_max_exclusive: &[u8; SEARCH_KEY_SIZE],
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError>;

    /// Get the number of pages used by the index. These pages may be empty, but the index should
    /// have allocated them and considers them available for use. Used for testing purposes.
    fn get_pages_used(&self) -> usize;
}
