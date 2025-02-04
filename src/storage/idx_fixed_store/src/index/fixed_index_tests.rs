// For tests provided by instructor. Add your own tests to fixed_index_file/page.rs
#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::buffer_pool::{BufferPool, BufferPoolTrait};
    use crate::index::fixed_index_file::FixedIndexFile;
    use crate::index::fixed_index_trait::IndexFileTrait;
    use crate::index::STARTING_PAGE_CAPACITY;
    use crate::prelude::SEARCH_KEY_SIZE;
    use common::ids::{ContainerId, StateType, TransactionId, ValueId};
    use common::testutil::init;
    use txn_manager::lm_trait::LockManagerTrait;
    use txn_manager::lockmanager::LockManager;

    fn set_up_test_util() -> (Arc<BufferPool>, Arc<LockManager>, TransactionId, bool, ContainerId) {
        init();
        let lm = Arc::new(LockManager::new(100));
        let bp = Arc::new(BufferPool::new(lm.clone()));
        let c_id = bp.register_container(None, StateType::HashTable).expect("Got CID");
        let txn = TransactionId::new();
        let is_range = match crate::index::INDEX_TYPE {
            StateType::Tree => true,
            StateType::HashTable => false,
            _ => panic!("Unsupported index type"),
        };
        (bp, lm, txn, is_range, c_id)

    }

    #[test]
    fn test_single_key_no_dupes() {
        let (bp, lm, txn, is_range, idx_c_id) = set_up_test_util();
        let idx = FixedIndexFile::new(idx_c_id, bp.clone(), lm.clone(), is_range, STARTING_PAGE_CAPACITY);
        let key = [1u8; SEARCH_KEY_SIZE];
        let v_id = ValueId::new_slot(1, 0, 0);
        let value_pointer = v_id.to_fixed_bytes();
        assert!(idx.add(&key, &value_pointer, &txn).is_ok());
        let pointers = idx.get_pointers_for_key(&key, &txn).expect("Should have pointers");
        assert_eq!(1, pointers.len());
        assert!(pointers.contains(&v_id));

        let key_no_match = [7u8; SEARCH_KEY_SIZE];
        let pointers = idx.get_pointers_for_key(&key_no_match, &txn).expect("Should have pointers");
        assert_eq!(0, pointers.len());
    }


    #[test]
    fn test_two_keys_no_dupes() {
        let (bp, lm, txn, is_range, idx_c_id) = set_up_test_util();
        let idx = FixedIndexFile::new(idx_c_id, bp.clone(), lm.clone(), is_range, STARTING_PAGE_CAPACITY);
        let key1 = [1u8; SEARCH_KEY_SIZE];
        let key2 = [2u8; SEARCH_KEY_SIZE];
        let v_id1 = ValueId::new_slot(1, 0, 0);
        let v_id2 = ValueId::new_slot(1, 0, 2);
        assert!(idx.add(&key1, &v_id1.to_fixed_bytes(), &txn).is_ok());
        assert!(idx.add(&key2, &v_id2.to_fixed_bytes(), &txn).is_ok());
        let pointers = idx.get_pointers_for_key(&key1, &txn).expect("Should have pointers");
        assert_eq!(1, pointers.len());
        assert!(pointers.contains(&v_id1));

        let pointers = idx.get_pointers_for_key(&key2, &txn).expect("Should have pointers");
        assert_eq!(1, pointers.len());
        assert!(pointers.contains(&v_id2));
    }

    #[test]
    fn test_one_key_dupes() {
        let (bp, lm, txn, is_range, idx_c_id) = set_up_test_util();
        let idx = FixedIndexFile::new(idx_c_id, bp.clone(), lm.clone(), is_range, STARTING_PAGE_CAPACITY);
        let key = [1u8; SEARCH_KEY_SIZE];
        let v_id1 = ValueId::new_slot(1, 0, 0);
        let v_id2 = ValueId::new_slot(1, 0, 2);
        assert!(idx.add(&key, &v_id1.to_fixed_bytes(), &txn).is_ok());
        assert!(idx.add(&key, &v_id2.to_fixed_bytes(), &txn).is_ok());
        let pointers = idx.get_pointers_for_key(&key, &txn).expect("Should have pointers");
        assert_eq!(2, pointers.len());
        assert!(pointers.contains(&v_id1));
        assert!(pointers.contains(&v_id2));
    }

}
