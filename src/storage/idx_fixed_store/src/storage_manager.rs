use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use common::prelude::*;
use txn_manager::{lm_trait::LockManagerTrait, lockmanager::LockManager};

use crate::{
    buffer_pool::{BufferPool, BufferPoolTrait},
    heap::fixed_heap_file::FixedHeapFile,
    index::{fixed_index_file::FixedIndexFile, fixed_index_trait::IndexFileTrait},
    prelude::{extract_search_key, SEARCH_KEY_SIZE},
};

type ResultKVs = Result<Vec<(Vec<u8>, Vec<u8>)>, CrustyError>;

struct Catalog<T: BufferPoolTrait> {
    tables: HashMap<ContainerId, FixedHeapFile<T>>,
    indexes: HashMap<ContainerId, FixedIndexFile<T>>,
    table_to_index: HashMap<ContainerId, ContainerId>,
}

struct StorageManager {
    lm: Arc<LockManager>,
    bp: Arc<BufferPool>,
    data_files: Arc<RwLock<Catalog<BufferPool>>>,
    //TODO milestone idx1 Add any fields you need here
}

impl StorageManager {
    pub fn new(timeout_ms: u64) -> Self {
        let lm = Arc::new(LockManager::new(timeout_ms));
        let bp = Arc::new(BufferPool::new(lm.clone()));
        let catalog = Catalog {
            tables: HashMap::new(),
            indexes: HashMap::new(),
            table_to_index: HashMap::new(),
        };
        let data_files = Arc::new(RwLock::new(catalog));
        StorageManager { lm, bp, data_files }
    }

    fn create_table_with_idx(
        &self,
        name: Option<String>,
    ) -> Result<(ContainerId, ContainerId), CrustyError> {
        let i_name = name.as_ref().map(|n| format!("{}_idx", n));
        let mut data_files = self.data_files.write().unwrap();
        let t_id = self.bp.register_container(name, StateType::BaseTable)?;
        let i_id = self
            .bp
            .register_container(i_name, crate::index::INDEX_TYPE)?;
        data_files.tables.insert(
            t_id,
            FixedHeapFile::new(t_id, self.bp.clone(), self.lm.clone()),
        );
        match crate::index::INDEX_TYPE {
            StateType::HashTable => {
                data_files.indexes.insert(
                    i_id,
                    FixedIndexFile::new(
                        i_id,
                        self.bp.clone(),
                        self.lm.clone(),
                        false,
                        crate::index::STARTING_PAGE_CAPACITY,
                    ),
                );
            }
            StateType::Tree => {
                data_files.indexes.insert(
                    i_id,
                    FixedIndexFile::new(
                        i_id,
                        self.bp.clone(),
                        self.lm.clone(),
                        true,
                        crate::index::STARTING_PAGE_CAPACITY,
                    ),
                );
            }
            StateType::BaseTable => {
                return Err(CrustyError::InvalidOperation);
            }
            StateType::MatView => {
                return Err(CrustyError::InvalidOperation);
            }
        }
        data_files.table_to_index.insert(t_id, i_id);
        Ok((t_id, i_id))
    }

    fn insert_kv(
        &self,
        c_id: &ContainerId,
        key: &[u8],
        val: &[u8],
        txn: &TransactionId,
    ) -> Result<ValueId, CrustyError> {
        let search_key = extract_search_key(val);
        let data_files = self.data_files.read().unwrap();
        let table = data_files
            .tables
            .get(c_id)
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let index = data_files
            .indexes
            .get(&data_files.table_to_index[c_id])
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let v_id = table.insert_kv(key, val, txn)?;
        let _i_v_id = index.add(search_key, &v_id.to_fixed_bytes(), txn)?;
        Ok(v_id)
    }

    fn insert_kvs(
        &self,
        c_id: &ContainerId,
        recs: Vec<(&[u8], &[u8])>,
        txn: &TransactionId,
    ) -> Result<Vec<ValueId>, CrustyError> {
        let search_keys = recs.iter().map(|(_, v)| extract_search_key(v)).collect();
        let data_files = self.data_files.read().unwrap();
        let table = data_files
            .tables
            .get(c_id)
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let index = data_files
            .indexes
            .get(&data_files.table_to_index[c_id])
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let v_ids = table.bulk_insert_kv(&recs, txn)?;
        let mut v_id_bytes = Vec::new(); //v_ids.iter().map(|v_id| v_id.to_fixed_bytes()).collect();
        for v_id in &v_ids {
            let fixed_bytes = v_id.to_fixed_bytes();
            v_id_bytes.push(fixed_bytes);
        }
        let _i_v_id = index.bulk_add(search_keys, v_id_bytes, txn)?;
        Ok(v_ids)
    }

    fn get_kv_by_val_id(
        &self,
        c_id: &ContainerId,
        v_id: &ValueId,
        txn: &TransactionId,
    ) -> Result<(Vec<u8>, Vec<u8>), CrustyError> {
        let data_files = self.data_files.read().unwrap();
        let table = data_files
            .tables
            .get(c_id)
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        table.get_kv(v_id, txn)
    }

    fn get_kvs_by_search_key_equality(
        &self,
        c_id: &ContainerId,
        search_key: &[u8; SEARCH_KEY_SIZE],
        txn: &TransactionId,
    ) -> ResultKVs {
        let mut res = Vec::new();
        let data_files = self.data_files.read().unwrap();
        let table = data_files
            .tables
            .get(c_id)
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let index = data_files
            .indexes
            .get(&data_files.table_to_index[c_id])
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let v_ids = index.get_pointers_for_key(search_key, txn)?;
        for v_id in v_ids {
            let kv = table.get_kv(&v_id, txn)?;
            res.push(kv);
        }
        Ok(res)
    }

    fn get_kvs_by_search_key_range(
        &self,
        c_id: &ContainerId,
        search_key_min_inclusive: &[u8; SEARCH_KEY_SIZE],
        search_key_max_exclusive: &[u8; SEARCH_KEY_SIZE],
        txn: &TransactionId,
    ) -> ResultKVs {
        let mut res = Vec::new();
        let data_files = self.data_files.read().unwrap();
        let table = data_files
            .tables
            .get(c_id)
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let index = data_files
            .indexes
            .get(&data_files.table_to_index[c_id])
            .ok_or(CrustyError::ContainerDoesNotExist)?;
        let v_ids = index.get_pointers_for_key_range(
            search_key_min_inclusive,
            search_key_max_exclusive,
            txn,
        )?;
        for v_id in v_ids {
            let kv = table.get_kv(&v_id, txn)?;
            res.push(kv);
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use rand::{rngs::SmallRng, SeedableRng};

    use crate::test_util::{gen_records_ascending_keys, SearchKeyTypes};


    #[test]
    fn test_storage_manager_single_thread() {
        use super::*;
        use crate::prelude::{KEY_SIZE, VALUE_SIZE};

        let sm = StorageManager::new(1000);
        let txn = TransactionId::new();
        let (t_id, i_id) = sm.create_table_with_idx(Some("test_table".to_string())).unwrap();

        let n = 1000;
        let mut rng = SmallRng::seed_from_u64(23530);

        let recs = gen_records_ascending_keys(n, SearchKeyTypes::Card(100), &mut rng);
        let mut search_keys_to_vids: HashMap<[u8; SEARCH_KEY_SIZE], Vec<ValueId>> = HashMap::with_capacity(100);
        let mut search_key_to_keys:  HashMap<[u8; SEARCH_KEY_SIZE], Vec<Vec<u8>>> = HashMap::with_capacity(100);
        let mut v_ids_to_kv = HashMap::new();
        for (key, value) in &recs {
            let v_id = sm.insert_kv(&t_id, &key, &value, &txn).unwrap();
            assert!(v_ids_to_kv.insert(v_id, (key.to_vec(), value.to_vec())).is_none());
            let search_key = extract_search_key(value);
            search_keys_to_vids
                .entry(*search_key)
                .or_insert_with(Vec::new)
                .push(v_id);
            search_key_to_keys
                .entry(*search_key)
                .or_insert_with(Vec::new)
                .push(key.to_vec());
        }
        for (search_key, keys) in search_key_to_keys {
            let kvs = sm.get_kvs_by_search_key_equality(&t_id, &search_key, &txn).unwrap();
            assert_eq!(kvs.len(), keys.len());
            for (k, _v) in kvs {
                assert!(keys.contains(&k));
            }
        }
        // TODO re-add after adding locks to lock manager
        //assert!(sm.lm.release_all_locks(txn).is_ok());

    }
}
