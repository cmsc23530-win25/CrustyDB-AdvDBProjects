use std::sync::Arc;

use super::fixed_heap_page::HeapDataPage;
use crate::buffer_pool::BufferPoolTrait;
use common::ids::AtomicPageId;
use common::prelude::*;
use std::sync::atomic::Ordering::Relaxed;
use txn_manager::lockmanager::LockManager;

#[allow(dead_code)]
pub struct FixedHeapFile<T: BufferPoolTrait> {
    /// A reference to the buffer pool
    bp: Arc<T>,
    /// A reference to the lock manager
    lm: Arc<LockManager>,
    /// This container Id
    c_id: ContainerId,
    /// The largest page id in this container
    max_page: AtomicPageId,
    /// The page id of the first page with a free slot
    free_page_cache: AtomicPageId,
}

#[allow(dead_code)]
impl<T: BufferPoolTrait> FixedHeapFile<T> {
    pub fn new(c_id: ContainerId, bp: Arc<T>, lm: Arc<LockManager>) -> Self {
        //Create first page. Assume that container has been registered
        //TODO milestone idx2 - Check LM first
        let (p_id, page) = bp.new_page(c_id).unwrap();
        assert_eq!(p_id, 0);
        drop(page);
        FixedHeapFile {
            bp,
            lm,
            c_id,
            max_page: AtomicPageId::new(0),
            free_page_cache: AtomicPageId::new(0),
        }
    }

    pub fn bulk_insert_kv(
        &self,
        key_values: &[(&[u8], &[u8])],
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError> {
        // This is a poor/simple implementation. It should be batched.
        let mut v_ids = Vec::new();
        for (key, value) in key_values.iter() {
            let v_id = self.insert_kv(key, value, txn)?;
            v_ids.push(v_id);
        }
        Ok(v_ids)
    }

    pub fn insert_kv(
        &self,
        key: &[u8],
        val: &[u8],
        _txn: &TransactionId,
    ) -> Result<ValueId, CrustyError> {
        loop {
            //TODO milestone idx2 - Check LM first
            let page_to_try = self.free_page_cache.load(Relaxed);
            let mut page_id = ValueId::new_page(self.c_id, page_to_try);
            let mut page = self
                .bp
                .get_page(&page_id, Permissions::ReadWrite)
                .expect("Error getting page");
            let slot = page.add(key, val);
            // Don't hold guard/latch long
            drop(page);
            match slot {
                Some(s_id) => {
                    page_id.slot_id = Some(s_id);
                    return Ok(page_id);
                }
                None => {
                    // There was no space
                    // How do we ensure no one else bumps and adds the new page?
                    //TODO milestone idx2 - Check LM first
                    if page_to_try == self.max_page.load(Relaxed) {
                        // need to make a new page
                        let new_page_id = page_to_try + 1;
                        let update = self.max_page.compare_exchange(
                            page_to_try,
                            new_page_id,
                            Relaxed,
                            Relaxed,
                        );
                        if update.is_ok() {
                            // We need to make the new page
                            self.bp.new_page(self.c_id).unwrap();
                            // Update the free page cache so we try this page next time
                            self.free_page_cache.store(new_page_id, Relaxed);
                        } // if else someone else added the page
                    } else {
                        let _ = self.free_page_cache.compare_exchange(
                            page_to_try,
                            page_to_try + 1,
                            Relaxed,
                            Relaxed,
                        );
                    }
                }
            }
        }
    }

    pub fn get_kv(
        &self,
        v_id: &ValueId,
        _txn: &TransactionId,
    ) -> Result<(Vec<u8>, Vec<u8>), CrustyError> {
        //TODO milestone idx2 Check LM first
        match self.bp.get_page(v_id, Permissions::ReadOnly) {
            Ok(page) => {
                let (k, v) = page.get_kv(v_id.slot_id.unwrap()).unwrap();
                Ok((k, v))
            }
            Err(e) => {
                error!("Error getting page {:?}", e);
                Err(e)
            }
        }
    }

    pub fn update_kv(
        &self,
        v_id: &ValueId,
        key: &[u8],
        val: &[u8],
        _txn: &TransactionId,
    ) -> Result<(), CrustyError> {
        //TODO milestone idx2  Check LM first
        let mut page = self
            .bp
            .get_page(v_id, Permissions::ReadWrite)
            .expect("Error getting page");
        match page.write(v_id.slot_id.unwrap(), true, key, val) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn delete_kv(&self, v_id: &ValueId, _txn: &TransactionId) -> Result<(), CrustyError> {
        //TODO milestone idx2  Check LM first
        let mut page = self
            .bp
            .get_page(v_id, Permissions::ReadWrite)
            .expect("Error getting page");
        page.delete(v_id.slot_id.unwrap());
        self.free_page_cache
            .fetch_min(v_id.page_id.unwrap(), Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::buffer_pool::BufferPool;
    use crate::prelude::*;
    use common::testutil::init;
    use txn_manager::lm_trait::LockManagerTrait;

    use super::*;

    #[test]
    fn test_data_file() {
        init();
        let lm = Arc::new(LockManager::new(500));
        let bp = Arc::new(BufferPool::new(lm.clone()));
        let txn = TransactionId::new();
        let c_id = bp
            .register_container(None, StateType::BaseTable)
            .expect("Got CID");
        let file = FixedHeapFile::new(c_id, bp.clone(), lm.clone());

        let mut key_values = Vec::new();
        for i in 0..DATA_VALUE_COUNT * 4 {
            let key = vec![i as u8; KEY_SIZE];
            let value = vec![i as u8; VALUE_SIZE];
            key_values.push((key, value));
        }

        let mut v_ids = Vec::new();
        for (key, value) in key_values.iter() {
            let v_id = file.insert_kv(&key, &value, &txn).unwrap();
            v_ids.push(v_id);
        }

        for (i, v_id) in v_ids.iter().enumerate() {
            let data = file.get_kv(&v_id, &txn);
            assert_eq!(data.unwrap(), key_values[i]);
        }

        //delete some values
        let d1_i = 3;
        let d2_i = 7 + DATA_VALUE_COUNT;

        let d1 = v_ids[d1_i];
        assert_eq!(0, d1.page_id.unwrap());

        let d2 = v_ids[d2_i];
        assert_eq!(1, d2.page_id.unwrap());

        assert!(file.delete_kv(&d1, &txn).is_ok());
        assert!(file.delete_kv(&d2, &txn).is_ok());

        v_ids.remove(d1_i);
        v_ids.remove(d2_i - 1);
        key_values.remove(d1_i);
        key_values.remove(d2_i - 1);

        for (i, v_id) in v_ids.iter().enumerate() {
            let data = file.get_kv(&v_id, &txn);
            if data.is_err() {
                panic!("Failed at index {} vid {:?}", i, v_id);
            }
            assert_eq!(data.unwrap(), key_values[i], "Failed at index {}", i);
        }

        let new_key = [u8::MAX; KEY_SIZE];
        let new_val = [u8::MAX; VALUE_SIZE];
        let new_v1 = file.insert_kv(new_key.as_ref(), new_val.as_ref(), &txn);
        assert_eq!(
            new_v1.unwrap(),
            d1,
            "Inserted value should take the place of deleted value"
        );
        let new_v2 = file.insert_kv(new_key.as_ref(), new_val.as_ref(), &txn);
        assert_eq!(
            new_v2.unwrap(),
            d2,
            "Inserted value should take the place of deleted value"
        );
    }
}
