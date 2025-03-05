use criterion::SamplingMode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use dictionary::dictionary_trait::dictionary_factory;
use dictionary::dictionary_trait::DictEncoding;
use dictionary::encode::convert_file_wrapper;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;


use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub struct ExperimentSample {
    data: Vec<Vec<u8>>,
    raw_size: usize,
    values_to_read: Vec<Vec<u8>>,
    keys_to_read: Vec<usize>,
    ranges_to_read: Vec<(Vec<u8>, Vec<u8>)>,
}

const SAMPLE_SIZE: usize = 10000;
const KEY_OPS: usize = 2000;
const VALUE_OPS: usize = 2000;
const RANGE_OPS: usize = 2000;


impl ExperimentSample {
    pub fn new(data_source: &Vec<Vec<u8>>, rng: &mut rand::rngs::SmallRng) -> Self {
        let mut data: Vec<Vec<u8>> = data_source.choose_multiple(rng, SAMPLE_SIZE).cloned().collect();
        let values_to_read = data.choose_multiple(rng, VALUE_OPS).cloned().collect();
        let offsets = (0..data.len()).collect::<Vec<usize>>();
        let keys_to_read = offsets.choose_multiple(rng, KEY_OPS).cloned().collect();
        data.sort();
        let mut ranges_to_read = Vec::new(); {
            for _ in 0..RANGE_OPS {
                let start = rng.gen_range(0..data.len()-10);
                let end = start + rng.gen_range(0..10);
                ranges_to_read.push((data[start].clone(), data[end].clone()));
            }
        }
        let raw_size = data.iter().map(|v| v.len()).sum();
        data.shuffle(rng);
        Self {
            data,
            raw_size,
            values_to_read,
            keys_to_read,
            ranges_to_read,
        }
    } 
}

pub fn run_benchmark(size_file: &mut File, dict_type: DictEncoding, exp: &ExperimentSample) {
    let dict = dictionary_factory(dict_type.clone(), &exp.data).unwrap();
    for key in &exp.keys_to_read {
        let _ = dict.decode_key(*key);
    }
    for value in &exp.values_to_read {
        let _ = dict.get_key(value);
    }
    for range in &exp.ranges_to_read {
        let _ = dict.get_key_range(&range.0, &range.1);
    }
    // Write the size to a CSV file 
    let encoded_size = dict.get_size_of_dictionary_encoded_array();
    //writeln!(size_file, "type,raw_size,encoded_size,records").unwrap();
    writeln!(size_file, "{:?},{},{},{}",dict_type,exp.raw_size,encoded_size,exp.data.len()).unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let f = "sample-100k.txt";
    const PERMUTATIONS: usize = 50;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(23530);
    let data = convert_file_wrapper(f).unwrap();

    let samples: [ExperimentSample; PERMUTATIONS] = std::array::from_fn(|_| ExperimentSample::new(&data, &mut rng));

    let mut size_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("dict_size.csv")
        .unwrap();
    writeln!(size_file, "type,raw_size,encoded_size,records").unwrap();

    let mut group = c.benchmark_group("flat-dict-sampling");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    group.bench_function("ArrayOP", |b| b.iter(|| {
        let exp = samples.choose(&mut rng).unwrap();
        run_benchmark(&mut size_file, DictEncoding::Array, exp);
    }));

    group.bench_function("Dense", |b| b.iter(|| {
        let exp = samples.choose(&mut rng).unwrap();
        run_benchmark(&mut size_file, DictEncoding::Dense, exp);
    }));

    group.bench_function("RePair", |b| b.iter(|| {
        let exp = samples.choose(&mut rng).unwrap();
        run_benchmark(&mut size_file, DictEncoding::RePair, exp);
    }));

    group.bench_function("Front", |b| b.iter(|| {
        let exp = samples.choose(&mut rng).unwrap();
        run_benchmark(&mut size_file, DictEncoding::Front, exp);
    }));

    group.bench_function("RePairFront", |b| b.iter(|| {
        let exp = samples.choose(&mut rng).unwrap();
        run_benchmark(&mut size_file, DictEncoding::RePairFront, exp);
    }));

    group.finish();

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);