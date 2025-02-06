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
    /// If needed, for a structure to identify the next page.
    pub page_pointer: PagePointer,
    /// If needed, an overflow page pointer
    pub overflow_pointer: PagePointer,
    /// If needed, a flag indicating if an index page is a leaf or not
    pub is_leaf: bool,
    /// If needed, a usize to use for any reason
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
            overflow_pointer: None,
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
        self.slot_capacity = min(PAGE_SLOT_LIMIT, (PAGE_SIZE) / (self.pair_size) ) as SlotId;
        assert!(
            self.slot_capacity > 6,
            "Page must hold at least 6 elements"
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

    /// Shift all records to the right starting from the given slot if possible.
    /// This shifts the records one place to the right up until the first empty slot encountered
    /// If there are empty slots to the right 
    /// Returns true if the shift was successful, false otherwise.
    /// If the slot is empty or out of bounds, an error is returned.
    pub fn shift_all_right(&mut self, slot: SlotId) -> Result<bool, CrustyError> {
        if slot >= self.slot_capacity {
            return Err(CrustyError::CrustyError("Slot out of bounds".to_string()));
        }
        let slot = slot as usize;
        if self.free[slot] {
            return Err(CrustyError::CrustyError("Slot is empty".to_string()));
        }
        // see if there is a free slot to the right
        for i in (slot + 1)..self.slot_capacity as usize {
            if self.free[i] {
                let mut buf = vec![];
                let data_shift_start = slot * self.pair_size;
                let data_shift_end = i * self.pair_size;
                buf.extend_from_slice(&self.data[data_shift_start..data_shift_end]);
                let os = self.pair_size;
                self.data[data_shift_start+os.. data_shift_end+os].copy_from_slice(&buf);
                self.free[slot] = true;
                self.free[i] = false;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// How many slots are filled
    pub fn get_filled_slot_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.slot_capacity {
            if !self.free[i as usize] {
                count += 1;
            }
        }
        count
    }

    /// How many slots are free
    pub fn get_free_slot_count(&self) -> usize {
        self.slot_capacity as usize - self.get_filled_slot_count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::{KEY_SIZE, VALUE_SIZE};

    #[test] 
    fn test_shift() {
        const BIG_SIZE: usize = 256;
        // should hold 8 KV pairs
        let mut p = FixedPage::new(1, BIG_SIZE, BIG_SIZE);
        assert_eq!(p.slot_capacity, 8);
        assert_eq!(p.get_filled_slot_count(), 0);
        assert_eq!(p.get_free_slot_count(), 8);
        let k1 = [1; BIG_SIZE];
        let k2 = [3; BIG_SIZE];
        let k3 = [4; BIG_SIZE];
        p.write(0, false, &k1, &k1).unwrap();
        p.write(1, false, &k2, &k2).unwrap();
        p.write(2, false, &k3, &k3).unwrap();
        assert_eq!(p.get_filled_slot_count(), 3);
        assert_eq!(p.get_free_slot_count(), 5);

        let (k, v) = p.get_kv(0).unwrap();
        assert_eq!(k, k1);
        assert_eq!(v, k1);

        let (k, v) = p.get_kv(1).unwrap();
        assert_eq!(k, k2);
        assert_eq!(v, k2);
        
        let (k, v) = p.get_kv(2).unwrap();
        assert_eq!(k, k3);
        assert_eq!(v, k3);

        // Shift all right from 1
        assert!(p.get_kv(3).is_none());
        assert_eq!(p.shift_all_right(1).unwrap(), true);

        // Slot counts should be the same
        assert_eq!(p.get_filled_slot_count(), 3);
        assert_eq!(p.get_free_slot_count(), 5);

        // Slot 0 should be in the same place
        let (k, v) = p.get_kv(0).unwrap();
        assert_eq!(k, k1);
        assert_eq!(v, k1);

        // 1 should be empty
        assert!(p.get_kv(1).is_none());

        // Slot 1 should be in slot 2
        let (k, v) = p.get_kv(2).unwrap();
        assert_eq!(k, k2);
        assert_eq!(v, k2);
        
        let (k, v) = p.get_kv(3).unwrap();
        assert_eq!(k, k3);
        assert_eq!(v, k3);

        assert!(p.get_kv(4).is_none());

        // move 3 to 4, making 3 empty
        assert!(p.move_if_empty(3,4).is_ok());
        assert!(p.get_kv(3).is_none());

        let (k, v) = p.get_kv(4).unwrap();
        assert_eq!(k, k3);
        assert_eq!(v, k3);

        // Shift right from 0. should only move 0 to 1 since it is empty
        assert_eq!(p.shift_all_right(0).unwrap(), true);
        assert!(p.get_kv(0).is_none());
        assert!(p.get_kv(3).is_none());

        let (k, v) = p.get_kv(4).unwrap();
        assert_eq!(k, k3);
        assert_eq!(v, k3);

        let (k, v) = p.get_kv(2).unwrap();
        assert_eq!(k, k2);
        assert_eq!(v, k2);

        let (k, v) = p.get_kv(1).unwrap();
        assert_eq!(k, k1);
        assert_eq!(v, k1);

        // fill in the empty slots
        let k4 = [5; BIG_SIZE];
        let k6 = [6; BIG_SIZE];
        assert!(p.write(0, false, &k4, &k4).is_ok());
        assert!(p.write(3, false, &k4, &k4).is_ok());
        assert!(p.write(4, false, &k4, &k4).is_err()); // already filled
        assert!(p.write(5, false, &k4, &k4).is_ok());
        assert!(p.write(6, false, &k4, &k4).is_ok());
        assert!(p.write(7, false, &k6, &k6).is_ok());
        assert!(p.write(8, false, &k4, &k4).is_err()); // out of bounds

        assert_eq!(p.get_filled_slot_count(), 8);
        assert_eq!(p.get_free_slot_count(), 0);

        assert!(p.shift_all_right(0).unwrap() == false);
        assert!(p.shift_all_right(3).unwrap() == false);
        assert!(p.shift_all_right(7).unwrap() == false);

        let (k, v) = p.get_kv(7).unwrap();
        assert_eq!(k, k6);
        assert_eq!(v, k6);
        p.delete(7);

        assert!(p.shift_all_right(3).unwrap());
        let (k, v) = p.get_kv(7).unwrap();
        assert_eq!(k, k4);
        assert_eq!(v, k4);
        assert_eq!(p.get_free_slot_count(), 1);

        assert!(p.get_kv(3).is_none());
    }

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
