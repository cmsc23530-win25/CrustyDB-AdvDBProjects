use std::sync::Arc;

use crate::buffer_pool::{BufferPool, BufferPoolTrait};
use crate::prelude::{INDEX_POINTER_SIZE, SEARCH_KEY_SIZE};
use common::ids::{ContainerId, SlotId, StateType, TransactionId, ValueId};
use common::testutil::init;
use txn_manager::lm_trait::LockManagerTrait;
use txn_manager::lockmanager::LockManager;

pub fn set_up_test_util() -> (
    Arc<BufferPool>,
    Arc<LockManager>,
    TransactionId,
    bool,
    ContainerId,
) {
    init();
    let lm = Arc::new(LockManager::new(100));
    let bp = Arc::new(BufferPool::new(lm.clone()));
    let c_id = bp
        .register_container(None, StateType::HashTable)
        .expect("Got CID");
    let txn = TransactionId::new();
    let is_range = match crate::index::INDEX_TYPE {
        StateType::Tree => true,
        StateType::HashTable => false,
        _ => panic!("Unsupported index type"),
    };
    (bp, lm, txn, is_range, c_id)
}

pub fn gen_unique_search_keys_and_value_ids(
    num: usize,
    slots_per_page: SlotId,
    c_id_for_v_id: ContainerId,
) -> Vec<([u8; SEARCH_KEY_SIZE], ValueId, [u8; INDEX_POINTER_SIZE])> {
    let mut res = Vec::new();
    let mut p_id = 0;
    let mut slot = 0;
    for i in 0..num {
        let key = [i as u8; SEARCH_KEY_SIZE];
        let v_id = ValueId::new_slot(c_id_for_v_id, p_id, slot);
        slot += 1;
        if slot == slots_per_page {
            slot = 0;
            p_id += 1;
        }
        res.push((key, v_id, v_id.to_fixed_bytes()));
    }
    res
}

// For tests provided by instructor. Add your own tests to fixed_index_file/page.rs
#[cfg(test)]
mod test {

    use crate::index::fixed_index_file::FixedIndexFile;
    use crate::index::fixed_index_tests::set_up_test_util;
    use crate::index::fixed_index_trait::IndexFileTrait;
    use crate::index::STARTING_PAGE_CAPACITY;
    use crate::prelude::SEARCH_KEY_SIZE;
    use common::ids::ValueId;
    use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

    use super::gen_unique_search_keys_and_value_ids;

    #[test]
    fn test_single_key_no_dupes() {
        let (bp, lm, txn, is_range, idx_c_id) = set_up_test_util();
        let idx = FixedIndexFile::new(
            idx_c_id,
            bp.clone(),
            lm.clone(),
            is_range,
            STARTING_PAGE_CAPACITY,
        );
        let key = [1u8; SEARCH_KEY_SIZE];
        let v_id = ValueId::new_slot(1, 0, 0);
        let value_pointer = v_id.to_fixed_bytes();
        assert!(idx.add(&key, &value_pointer, &txn).is_ok());
        let pointers = idx
            .get_pointers_for_key(&key, &txn)
            .expect("Should have pointers");
        assert_eq!(1, pointers.len());
        assert!(pointers.contains(&v_id));

        let key_no_match = [7u8; SEARCH_KEY_SIZE];
        let pointers = idx
            .get_pointers_for_key(&key_no_match, &txn)
            .expect("Should have pointers");
        assert_eq!(0, pointers.len());
    }

    #[test]
    fn test_two_keys_no_dupes() {
        let (bp, lm, txn, is_range, idx_c_id) = set_up_test_util();
        let idx = FixedIndexFile::new(
            idx_c_id,
            bp.clone(),
            lm.clone(),
            is_range,
            STARTING_PAGE_CAPACITY,
        );
        let key1 = [1u8; SEARCH_KEY_SIZE];
        let key2 = [2u8; SEARCH_KEY_SIZE];
        let v_id1 = ValueId::new_slot(1, 0, 0);
        let v_id2 = ValueId::new_slot(1, 0, 2);
        assert!(idx.add(&key1, &v_id1.to_fixed_bytes(), &txn).is_ok());
        assert!(idx.add(&key2, &v_id2.to_fixed_bytes(), &txn).is_ok());
        let pointers = idx
            .get_pointers_for_key(&key1, &txn)
            .expect("Should have pointers");
        assert_eq!(1, pointers.len());
        assert!(pointers.contains(&v_id1));

        let pointers = idx
            .get_pointers_for_key(&key2, &txn)
            .expect("Should have pointers");
        assert_eq!(1, pointers.len());
        assert!(pointers.contains(&v_id2));
    }

    #[test]
    fn test_one_key_dupes() {
        let (bp, lm, txn, is_range, idx_c_id) = set_up_test_util();
        let idx = FixedIndexFile::new(
            idx_c_id,
            bp.clone(),
            lm.clone(),
            is_range,
            STARTING_PAGE_CAPACITY,
        );
        let key = [1u8; SEARCH_KEY_SIZE];
        let v_id1 = ValueId::new_slot(1, 0, 0);
        let v_id2 = ValueId::new_slot(1, 0, 2);
        assert!(idx.add(&key, &v_id1.to_fixed_bytes(), &txn).is_ok());
        assert!(idx.add(&key, &v_id2.to_fixed_bytes(), &txn).is_ok());
        let pointers = idx
            .get_pointers_for_key(&key, &txn)
            .expect("Should have pointers");
        assert_eq!(2, pointers.len());
        assert!(pointers.contains(&v_id1));
        assert!(pointers.contains(&v_id2));
    }

    #[test]
    fn test_gen() {
        let (bp, lm, txn, is_range, idx_c_id) = set_up_test_util();
        // Create Index
        let idx = FixedIndexFile::new(
            idx_c_id,
            bp.clone(),
            lm.clone(),
            is_range,
            STARTING_PAGE_CAPACITY,
        );
        // Create keys and value ids and bytes of value ids
        let mut recs = gen_unique_search_keys_and_value_ids(200, 32, 99);

        // Let's shuffle the records
        let mut rng = SmallRng::seed_from_u64(23530);
        recs.shuffle(&mut rng);

        // Add records to index
        for (key, _v_id, pointer) in recs.iter() {
            assert!(idx.add(key, pointer, &txn).is_ok());
        }

        // Let's shuffle the list and check
        recs.shuffle(&mut rng);

        for (key, v_id, _pointer) in recs.iter() {
            let pointers = idx
                .get_pointers_for_key(key, &txn)
                .expect("Should have pointers");
            assert_eq!(pointers.len(), 1);
            assert!(pointers.contains(v_id));
        }
    }
}