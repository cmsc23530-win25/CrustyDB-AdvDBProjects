use crate::fixed_page::FixedPage;
use crate::prelude::*;
use common::prelude::*;

pub trait HeapDataPage {
    fn new(p_id: PageId) -> Self;
    fn add(&mut self, key: &[u8], value: &[u8]) -> Option<SlotId>;
}

impl HeapDataPage for FixedPage {
    fn new(p_id: PageId) -> Self {
        FixedPage::new(p_id, KEY_SIZE, VALUE_SIZE)
    }

    fn add(&mut self, key: &[u8], value: &[u8]) -> Option<SlotId> {
        let first_slot = self.free.iter().position(|&x| x);
        match first_slot {
            Some(s) => {
                let slot = s as SlotId;
                if self.write(slot, false, key, value).is_err() {
                    None
                } else {
                    Some(slot)
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fixed_page::FixedPage;
    use crate::test_util;
    use common::testutil::init;

    #[test]
    fn sample_util_page_test() {
        // Call init to set up logger. Default is info.
        // Change init() to debug to see debug logs, etc
        init();
        // Construct a page with page_id 1
        // Using type inference to avoid specifying the type of page
        let mut page = <FixedPage as HeapDataPage>::new(1);

        // Calling the testutil to generate N tuples of key and value
        // Specifically, the keys are ascending
        // The values are random except for the last SEARCH_KEY_SIZE bytes
        // which are the search key and are controlled by the SearchKeyTypes enum/
        // The last argument is the RNG to generate random numbers. Use from_entropy
        // to seed the RNG with entropy or from_seed to seed with a fixed seed and
        // reproduce the same results every time.
        let record_count = DATA_VALUE_COUNT; // expected data value count
        let search_keys = test_util::SearchKeyTypes::Card(5); // The search key is one of 5 values
        let mut rng = test_util::gen_small_rng_with_seed(23530);
        let records = test_util::gen_records_ascending_keys(record_count, search_keys, &mut rng);
        for (i, (key, value)) in records.iter().enumerate() {
            let slot = page.add(key, value);
            assert!(slot.is_some());
            assert_eq!(slot.unwrap(), i as SlotId);
        }
    }

    #[test]
    fn test_data_page() {
        init();
        let mut page = <FixedPage as HeapDataPage>::new(1);
        let key = vec![0; KEY_SIZE];
        let value = vec![1; VALUE_SIZE];
        let slot = page.add(&key, &value);
        assert!(slot.is_some());
        let data = page.get_kv(slot.unwrap()).unwrap();
        assert_eq!(data.0, key);
        assert_eq!(data.1, value);

        let key2 = vec![2; KEY_SIZE];
        let value2 = vec![3; VALUE_SIZE];
        let slot2 = page.add(&key2, &value2);
        assert!(slot2.is_some());
        let data2 = page.get_kv(slot2.unwrap()).unwrap();
        assert_eq!(data2.0, key2);
        assert_eq!(data2.1, value2);

        page.delete(slot.unwrap());
        let data = page.get_kv(slot.unwrap());
        assert!(data.is_none());

        assert_eq!(page.get_kv(slot2.unwrap()).unwrap().1, value2);
    }

    #[test]
    fn filled_page() {
        let mut page = <FixedPage as HeapDataPage>::new(1);
        let mut values = Vec::new();
        for i in 0..DATA_VALUE_COUNT {
            let key = vec![i as u8; KEY_SIZE];
            let value = vec![i as u8; VALUE_SIZE];
            let slot = page.add(&key, &value);
            values.push((key, value));
            assert_eq!(slot.unwrap(), i as SlotId);
        }
        let slot = page.add(&values[0].0, &values[0].1);
        assert!(slot.is_none());

        for (i, value) in values.iter().enumerate() {
            assert_eq!(page.get_kv(i as SlotId).unwrap().1, *value.1);
        }
    }
}
