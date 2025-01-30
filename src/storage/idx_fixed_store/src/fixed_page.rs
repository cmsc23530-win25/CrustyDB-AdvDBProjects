use crate::prelude::*;
use common::prelude::*;
use std::cmp::min;

/// The fixed page struct for representing data and index pages for fixed size
/// records. Each record is assumed to have two parts a key and a value.
/// For the data page, the key is assumed (but not checked) to be unique and
/// the value is the data associated with the key. For the index page, the key
/// is the search key and the value is the pointer to a data or index page.
/// For simplicity we store the page metadata outside of the data array.
pub struct FixedPage {
    /// The PageId of this page
    pub p_id: PageId,
    /// The data array for the page's records
    pub data: [u8; PAGE_SIZE],
    /// How many slots can this page hold based on the key and value sizes
    pub slot_capacity: SlotId,
    /// The size of the key in bytes (same for all records)
    pub key_size: usize,
    /// The size of the value in bytes (same for all records)
    pub value_size: usize,
    /// The size of the key and value pair added together
    pub pair_size: usize,
    /// A boolean array to track which slots are free. This is larger than the
    /// slot_capacity as we need to statically allocate the array.
    pub free: [bool; PAGE_SLOT_LIMIT],
    /// If needed for a structure to identify the next page.
    pub page_pointer: PagePointer,
    /// If needed a flag indicating if an index page is a leaf or not
    pub is_leaf: bool,
    /// A usize to use if you need for any reason
    pub extra: usize,
}

impl FixedPage {
    pub fn empty() -> Self {
        FixedPage {
            p_id: 0,
            data: [0; PAGE_SIZE],
            slot_capacity: PAGE_SLOT_LIMIT as SlotId,
            key_size: 0,
            value_size: 0,
            pair_size: 0,
            free: [false; PAGE_SLOT_LIMIT],
            page_pointer: None,
            is_leaf: false,
            extra: 0,
        }
    }
    pub fn new(p_id: PageId, key_size: usize, value_size: usize) -> Self {
        let mut page = FixedPage::empty();
        page.update_settings(p_id, key_size, value_size);
        page
    }
    pub fn update_settings(&mut self, p_id: PageId, key_size: usize, value_size: usize) {
        self.p_id = p_id;
        self.key_size = key_size;
        self.value_size = value_size;
        self.pair_size = key_size + value_size;
        self.slot_capacity = min(PAGE_SLOT_LIMIT, (PAGE_SIZE) / (self.pair_size) - 1) as SlotId;
        assert!(
            self.slot_capacity > 10,
            "Page must hold at least 10 elements"
        );
        // Set it so invalid slots of false
        for i in 0..self.slot_capacity {
            self.free[i as usize] = true;
        }
    }

    pub fn write(
        &mut self,
        slot: SlotId,
        overwrite: bool,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), CrustyError> {
        if slot > self.slot_capacity {
            return Err(CrustyError::CrustyError(
                "Writing past slot capacity".to_string(),
            ));
        }
        let slot = slot as usize;
        if overwrite || self.free[slot] {
            let mut os = slot * self.pair_size;
            if self.key_size > 0 {
                self.data[os..os + self.key_size].copy_from_slice(key);
                os += self.key_size;
            }
            self.data[os..os + self.value_size].copy_from_slice(value);
            self.free[slot] = false;
            return Ok(());
        }
        Err(CrustyError::StorageError)
    }

    pub fn move_if_empty(&mut self, from_slot: SlotId, to_slot: SlotId) -> Result<(), CrustyError> {
        if from_slot > self.slot_capacity || to_slot > self.slot_capacity {
            return Err(CrustyError::CrustyError(
                "Writing past slot capacity".to_string(),
            ));
        }
        let from_slot = from_slot as usize;
        let to_slot = to_slot as usize;

        // Check if from is filled and to is empty
        if !self.free[from_slot] && self.free[to_slot] {
            let old_os = self.pair_size * from_slot;
            let new_os = self.pair_size * to_slot;

            let mut buf = vec![];
            buf.extend_from_slice(&self.data[old_os..old_os + self.pair_size]);
            self.data[new_os..new_os + self.pair_size].copy_from_slice(&buf);
            self.free[from_slot] = true;
            self.free[to_slot] = false;
            return Ok(());
        }
        Err(CrustyError::StorageError)
    }

    pub fn delete(&mut self, slot: SlotId) {
        if slot > self.slot_capacity {
            return;
        }
        self.free[slot as usize] = true;
    }

    pub fn delete_all(&mut self) {
        for i in 0..self.slot_capacity {
            self.free[i as usize] = true;
        }
    }

    pub fn get_kv(&self, slot: SlotId) -> Option<(Vec<u8>, Vec<u8>)> {
        if slot > self.slot_capacity {
            return None;
        }
        let slot = slot as usize;
        if self.free[slot] {
            return None;
        }
        let mut b1 = vec![];
        let mut b2 = vec![];
        let mut os = slot * self.pair_size;
        b1.extend_from_slice(&self.data[os..os + self.key_size]);
        os += self.key_size;
        b2.extend_from_slice(&self.data[os..os + self.value_size]);
        Some((b1, b2))
    }

    pub fn get_kv_pairs(&self) -> Vec<(SlotId, Vec<u8>, Vec<u8>)> {
        let mut res = vec![];
        for slot in 0..self.slot_capacity {
            if !self.free[slot as usize] {
                let (k, v) = self.get_kv(slot).unwrap();
                res.push((slot, k, v));
            }
        }
        res
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::{KEY_SIZE, VALUE_SIZE};

    #[test]
    fn test_page() {
        let k1 = [1; KEY_SIZE];
        let k2 = [3; KEY_SIZE];
        let k3 = [5; KEY_SIZE];
        let v1 = [2; VALUE_SIZE];
        let v2 = [4; VALUE_SIZE];
        let v3 = [6; VALUE_SIZE];
        let mut p = FixedPage::new(1, KEY_SIZE, VALUE_SIZE);
        assert_eq!(p.get_kv(0), None);
        assert!(p.write(0, false, &k1, &v1).is_ok());
        assert!(p.write(0, false, &k3, &v3).is_err());
        let (mut k, mut v) = p.get_kv(0).unwrap();
        assert_eq!(k, k1);
        assert_eq!(v, v1);
        assert!(p.write(1, false, &k2, &v2).is_ok());
        (k, v) = p.get_kv(1).unwrap();
        assert_eq!(k, k2);
        assert_eq!(v, v2);
        assert!(p.write(3, false, &k3, &v3).is_ok());
        let mut all = p.get_kv_pairs();
        assert_eq!(all[0].0, 0);
        assert_eq!(all[1].0, 1);
        assert_eq!(all[2].0, 3);
        assert_eq!(all[2].1, k3);
        assert_eq!(all[2].2, v3);
        assert!(p.move_if_empty(0, 1).is_err());
        assert!(p.move_if_empty(0, 2).is_ok());
        all = p.get_kv_pairs();
        assert_eq!(all[0].0, 1);
        assert_eq!(all[1].0, 2);
        assert_eq!(all[2].0, 3);
        assert_eq!(all[2].0, 3);
        assert_eq!(all[2].1, k3);
        assert_eq!(all[2].2, v3);
        p.delete(2);
        p.delete(3);
        all = p.get_kv_pairs();
        assert_eq!(all[0].0, 1);
        assert_eq!(all[0].1, k2);
        assert_eq!(all[0].2, v2);
    }
}
