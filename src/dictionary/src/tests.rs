#[cfg(test)]
mod tests {
    use common::testutil::init;
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use rand::SeedableRng;

    use crate::dictionary_trait::dictionary_factory;
    use crate::dictionary_trait::DictEncoding;
    use crate::dictionary_trait::OPDictionaryTrait;
    use crate::dictionary_trait::MAX_STRING_LENGTH;
    use crate::encode::convert_file_wrapper;

    const TEST_FILES: [&str; 5] = [
        "first100.txt",
        "first1000.txt",
        "sample-1000.txt",
        "sample-10k.txt",
        "sample-100k.txt",
    ];

    fn get_data_size(data: &Vec<Vec<u8>>) -> usize {
        let mut size = 0;
        for d in data {
            size += d.len();
        }
        size
    }

    #[test]
    fn test_simple_non_op() {
        init();
        let data = convert_file_wrapper(TEST_FILES[0]).unwrap();
        let d = dictionary_factory(DictEncoding::NonOPArray, &data).unwrap();
        assert!(d.get_size_of_dictionary_encoded_array() == get_data_size(&data));
        let mut rng = SmallRng::seed_from_u64(23530);
        test_non_op_keys(&d, &data, &mut rng);
    }

    #[test]
    fn test_all_files_non_op() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES[..2] {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::NonOPArray, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() == get_data_size(&data));
            test_non_op_keys(&d, &data, &mut rng);
        }
    }

    #[test]
    fn array_op_lookups() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Array, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() == data.len() * MAX_STRING_LENGTH);
            test_keys_random_order(&d, &data, &mut rng);
        }
    }

    #[test]
    fn array_op_range_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Array, &data).unwrap();
            test_existing_keys_range(d, &data, &mut rng);
        }
    }

    #[test]
    fn array_op_range_non_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Array, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() == data.len() * MAX_STRING_LENGTH);
            test_non_existing_keys_range(d, &data, &mut rng);
        }
    }

    #[test]
    fn dense_op_lookups() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Dense, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() == get_data_size(&data));
            test_keys_random_order(&d, &data, &mut rng);
        }
    }

    #[test]
    fn dense_op_range_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Dense, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() == get_data_size(&data));
            test_existing_keys_range(d, &data, &mut rng);
        }
    }
    
    #[test]
    fn dense_op_range_non_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Dense, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() == get_data_size(&data));
            test_non_existing_keys_range(d, &data, &mut rng);
        }
    }


    #[test]
    fn front_op_lookups() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Front, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_keys_random_order(&d, &data, &mut rng);
        }
    }

    #[test]
    fn front_op_range_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Front, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_existing_keys_range(d, &data, &mut rng);
        }
    }

    #[test]
    fn front_op_range_non_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::Front, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_non_existing_keys_range(d, &data, &mut rng);
        }
    }

    #[test]
    fn repair_op_lookups() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::RePair, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_keys_random_order(&d, &data, &mut rng);
        }
    }

    #[test]
    fn repair_op_range_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::RePair, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_existing_keys_range(d, &data, &mut rng);
        }
    }

    #[test]
    fn repair_op_range_non_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::RePair, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_non_existing_keys_range(d, &data, &mut rng);
        }
    }


    #[test]
    fn repair_front_op_lookups() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::RePairFront, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_keys_random_order(&d, &data, &mut rng);
        }
    }

    #[test]
    fn repair_front_op_range_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::RePairFront, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_existing_keys_range(d, &data, &mut rng);
        }
    }

    #[test]
    fn repair_front_op_range_non_existing() {
        init();
        let mut rng = SmallRng::seed_from_u64(23530);
        for file in &TEST_FILES {
            let data = convert_file_wrapper(file).unwrap();
            let d = dictionary_factory(DictEncoding::RePairFront, &data).unwrap();
            assert!(d.get_size_of_dictionary_encoded_array() < get_data_size(&data));
            test_non_existing_keys_range(d, &data, &mut rng);
        }
    }
    /// Only used for NON-OP dictionary given as reference
    fn test_non_op_keys(
        dict: &Box<dyn OPDictionaryTrait>,
        data: &Vec<Vec<u8>>,
        rng: &mut rand::rngs::SmallRng,
    ) {
        let mut offsets: Vec<usize> = (0..data.len()).collect();
        // To see values uncomment the following
        // for i in &offsets {
        //     trace!("Decode key failed at  index (orig / decoded) {}: {:10}|{:10}", i, String::from_utf8(data[*i].clone()).unwrap(), String::from_utf8(dict.decode_key(*i).clone()).unwrap());
        // }
        offsets.shuffle(rng);
        for i in &offsets {
            assert_eq!(
                dict.decode_key(*i),
                data[*i].as_slice(),
                "Decode key failed at  index (orig / decoded) {}: {:10}|{:10}",
                i,
                String::from_utf8(data[*i].clone()).unwrap(),
                String::from_utf8(dict.decode_key(*i).clone()).unwrap()
            );
            assert_eq!(
                dict.get_key(data[*i].as_slice()),
                Some(*i),
                "Get key failed at index {}",
                i
            );
        }
    }

    fn test_keys_random_order(
        dict: &Box<dyn OPDictionaryTrait>,
        data: &Vec<Vec<u8>>,
        rng: &mut rand::rngs::SmallRng,
    ) {
        let mut data_sorted = data.clone();
        data_sorted.sort();
        let mut offsets: Vec<usize> = (0..data.len()).collect();
        for i in &offsets {
            trace!(
                "Decode key failed at  index (orig / decoded) {}: {:10}|{:10}",
                i,
                String::from_utf8(data_sorted[*i].clone()).unwrap(),
                String::from_utf8(dict.decode_key(*i).clone()).unwrap()
            );
        }
        offsets.shuffle(rng);
        for i in &offsets {
            assert_eq!(
                dict.decode_key(*i),
                data_sorted[*i].as_slice(),
                "Decode key failed at  index (orig / decoded) {}: {:10}|{:10}",
                i,
                String::from_utf8(data_sorted[*i].clone()).unwrap(),
                String::from_utf8(dict.decode_key(*i).clone()).unwrap()
            );
            assert_eq!(
                dict.get_key(data_sorted[*i].as_slice()),
                Some(*i),
                "Get key failed at index {}",
                i
            );
        }
    }


    fn test_non_existing_keys_range(
        dict: Box<dyn OPDictionaryTrait>,
        data: &Vec<Vec<u8>>,
        rng: &mut rand::rngs::SmallRng,
    ) {
        let mut sorted: Vec<Vec<u8>> = data.iter().map(|v| v.clone()).collect();
        sorted.sort();

        // Check min and max possible values give 0 and data.len()
        let min = [0u8; MAX_STRING_LENGTH];
        let max = [127u8; MAX_STRING_LENGTH];
        trace!(
            "Min|Max {:10}|{:10}",
            String::from_utf8(min.to_vec()).unwrap(),
            String::from_utf8(max.to_vec()).unwrap()
        );
        let (start, end) = dict.get_key_range(&min, &max);
        assert_eq!(start, 0);
        assert_eq!(end, data.len());

        // Loop and check random key ranges that do not exist in the dictionary
        // start with end that is not in the dictionary
        trace!(" ---  Checking random key ranges for existing start and non-existing end keys");
        for _ in 0..20 {
            let start = rng.gen_range(0..data.len() - 1);
            let end = rng.gen_range(start..data.len());
            let mut end_key = sorted[end].clone();
            let last_pos = end_key.len() - 1;
            let mut last_char = end_key[last_pos];
            if (last_char as u8) < 127 {
                last_char += 1;
                end_key[last_pos] = last_char;
            } else {
                continue;
            }
            if data.contains(&end_key) {
                continue;
            }
            if end < sorted.len() - 1 && end_key > sorted[end + 1] {
                // simple way of modifying string cannot guarantee its not smaller than next key.
                continue;
            }
            trace!(
                "Start|End  ({:2}){:10}|{:10}(modified from {:10} at {:2})",
                start,
                String::from_utf8(sorted[start].clone()).unwrap(),
                String::from_utf8(end_key.clone()).unwrap(),
                String::from_utf8(sorted[end].clone()).unwrap(),
                end
            );
            let (start_idx, end_idx) = dict.get_key_range(&sorted[start], &end_key);
            assert_eq!(
                start_idx, start,
                "Start index failed for start {} and end {}",
                start, end
            );
            assert_eq!(
                end_idx, end,
                "End index failed for start {} and end {}",
                start, end
            );
        }

        // Loop and check random key ranges that do not exist in the dictionary
        // start key not existing
        trace!(" ---  Checking random key ranges for non-existing start and existing end keys");
        for _ in 0..20 {
            let start = rng.gen_range(0..data.len() - 1);
            let end = rng.gen_range(start..data.len());
            let mut start_key = sorted[start].clone();
            let last_pos = start_key.len() - 1;
            let mut last_char = start_key[last_pos];
            if (last_char as u8) > 0 {
                last_char -= 1;
                start_key[last_pos] = last_char;
            } else {
                continue;
            }
            if data.contains(&start_key) {
                continue;
            }
            if start > 0 && start_key < sorted[start - 1] {
                // simple way of modifying string cannot guarantee its not smaller than next key.
                continue;
            }
            trace!(
                "Start|End  ({:2}){:10}|{:10}({:2}) - start modified from {:10}",
                start,
                String::from_utf8(start_key.clone()).unwrap(),
                String::from_utf8(sorted[end].clone()).unwrap(),
                end,
                String::from_utf8(sorted[start].clone()).unwrap()
            );
            let (start_idx, end_idx) = dict.get_key_range(&start_key, &sorted[end]);
            assert_eq!(
                start_idx, start,
                "Start index failed for start {} and end {}",
                start, end
            );
            assert_eq!(
                end_idx, end,
                "End index failed for start {} and end {}",
                start, end
            );
        }
    }

    fn test_existing_keys_range(
        dict: Box<dyn OPDictionaryTrait>,
        data: &Vec<Vec<u8>>,
        rng: &mut rand::rngs::SmallRng,
    ) {
        let mut sorted: Vec<Vec<u8>> = data.iter().map(|v| v.clone()).collect();
        sorted.sort();

        trace!(" ---  Checking random key ranges for existing keys");
        // Loop and check random key ranges that exist in the dictionary
        for _ in 0..20 {
            let start = rng.gen_range(0..data.len() - 1);
            let end = rng.gen_range(start..data.len());
            trace!(
                "Start|End  ({:2}){:10}|{:10}({:2})",
                start,
                String::from_utf8(sorted[start].clone()).unwrap(),
                String::from_utf8(sorted[end].clone()).unwrap(),
                end
            );
            let (start_idx, end_idx) = dict.get_key_range(&sorted[start], &sorted[end]);
            assert_eq!(start_idx, start);
            assert_eq!(end_idx, end);
        }
    }
}
