use crate::prelude::*;
use rand::rngs::SmallRng;
use rand::{seq::SliceRandom, Rng, SeedableRng};

#[derive(PartialEq)]
pub enum SearchKeyTypes {
    Random,
    Card(usize),
    Distinct,
}

const VAL_SIZE_NO_SEARCH_KEY: usize = VALUE_SIZE - SEARCH_KEY_SIZE;

pub fn gen_small_rng_with_seed(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}
pub fn gen_records_ascending_keys(
    n: usize,
    search_key: SearchKeyTypes,
    rng: &mut SmallRng,
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut records = Vec::with_capacity(n);
    let mut outer_loop = 1;
    let mut inner_loop = n;

    if SEARCH_KEY_SIZE > 8 {
        panic!("Search key size is too large");
    }
    let mut search_key_candidates: Vec<Vec<u8>> = match search_key {
        SearchKeyTypes::Random => (0..n)
            .map(|_| rng.gen::<u64>().to_be_bytes().to_vec())
            .collect(),
        SearchKeyTypes::Card(card) => (0..card)
            .map(|_| rng.gen::<u64>().to_be_bytes().to_vec())
            .collect(),
        SearchKeyTypes::Distinct => (0..n).map(|i| i.to_be_bytes().to_vec()).collect(),
    };

    if search_key == SearchKeyTypes::Distinct {
        search_key_candidates.shuffle(rng);
    }

    let mut search_keys = Vec::with_capacity(n);
    for _ in 0..n {
        if search_key == SearchKeyTypes::Distinct {
            search_keys.push(search_key_candidates.pop().unwrap());
        } else {
            search_keys
                .push(search_key_candidates[rng.gen_range(0..search_key_candidates.len())].clone());
        }
    }

    if KEY_SIZE > 8 {
        outer_loop = KEY_SIZE / 8;
        inner_loop = n / outer_loop;
    }

    let mut counter = 0;
    for i in 0..outer_loop {
        for j in 0..inner_loop {
            let key = if KEY_SIZE > 8 {
                let mut key = i.to_be_bytes().to_vec();
                key.extend_from_slice(&j.to_be_bytes());
                key
            } else {
                j.to_be_bytes().to_vec()[..KEY_SIZE].to_vec()
            };
            let mut value: Vec<u8> = (0..VAL_SIZE_NO_SEARCH_KEY).map(|_| rng.gen()).collect();
            value.extend_from_slice(&search_keys[counter]);
            records.push((key, value));
            counter += 1;
        }
    }
    records
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_ascending_records() {
        let mut rng = SmallRng::from_entropy();
        let records = gen_records_ascending_keys(100, SearchKeyTypes::Random, &mut rng);
        let mut prev_key = vec![0; KEY_SIZE];
        assert_eq!(records.len(), 100);
        for (i, (key, value)) in records.iter().enumerate() {
            if i > 0 {
                assert!(key > &prev_key);
            }
            assert_eq!(key.len(), KEY_SIZE);
            assert_eq!(value.len(), VALUE_SIZE);
            prev_key = key.clone();
        }
    }

    #[test]
    fn test_ascending_records_card_search_key() {
        let mut rng = SmallRng::from_entropy();
        let card = 18;
        let n = 500;
        let records = gen_records_ascending_keys(n, SearchKeyTypes::Card(card), &mut rng);
        let mut prev_key = vec![0; KEY_SIZE];
        assert_eq!(records.len(), n);
        let mut search_keys: HashSet<Vec<u8>> = HashSet::new();
        for (i, (key, value)) in records.iter().enumerate() {
            if i > 0 {
                assert!(key > &prev_key);
            }
            assert_eq!(key.len(), KEY_SIZE);
            assert_eq!(value.len(), VALUE_SIZE);
            prev_key = key.clone();
            let search_key = value[VALUE_SIZE - SEARCH_KEY_SIZE..].to_vec();
            assert_eq!(search_key.len(), SEARCH_KEY_SIZE);
            search_keys.insert(search_key);
        }
        assert_eq!(search_keys.len(), card);
    }

    #[test]
    fn test_ascending_records_distinct_search_key() {
        let mut rng = SmallRng::from_entropy();
        let n = 500;
        let records = gen_records_ascending_keys(n, SearchKeyTypes::Distinct, &mut rng);
        let mut prev_key = vec![0; KEY_SIZE];
        assert_eq!(records.len(), n);
        let mut search_keys: HashSet<Vec<u8>> = HashSet::new();
        for (i, (key, value)) in records.iter().enumerate() {
            if i > 0 {
                assert!(key > &prev_key);
            }
            assert_eq!(key.len(), KEY_SIZE);
            assert_eq!(value.len(), VALUE_SIZE);
            prev_key = key.clone();
            let search_key = value[VALUE_SIZE - SEARCH_KEY_SIZE..].to_vec();
            assert_eq!(search_key.len(), SEARCH_KEY_SIZE);
            search_keys.insert(search_key);
        }
        assert_eq!(search_keys.len(), n);
    }
}
