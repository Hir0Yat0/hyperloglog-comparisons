use std::io::{self, Write};

use rand::{rngs::ThreadRng, seq::index};

use crate::vanilla::hll::*;

pub fn run_benchmark_speed(){
    use std::time::{Duration, Instant};
    use rand::prelude::*;
    let num_bucket_bits = 12;
    let mut hll = HLL::new(num_bucket_bits);
    let mut rng = rand::rng();
    let time_start_1 = Instant::now();
    // let input_stream = {
        
    //     (0u64..1_000_000_000u64).map(|_i| rng.next_u64()).into_iter().map(|_i| rng.next_u64())
    // };
    // hll.read_stream(&Box::new(input_stream));
    for _i in 0u64..1_000_000_000u64 {
        hll.read_data(rng.random());
    }
    let time_start_1_elapsed_1 = time_start_1.elapsed();
    println!("Finished Reading Stream and Counting in {} secs", time_start_1_elapsed_1.as_nanos() as f64 * 1e-9f64);
    let time_start_2 = Instant::now();
    let results = hll.get_cardinality();
    let time_start_2_elapsed_1 = time_start_2.elapsed();
    println!("Finished Cardinality Estimatings in {} secs with results = {}!", time_start_2_elapsed_1.as_nanos() as f64 * 1e-9f64, results);
}

pub fn run_benchmark_accuracy() {
    use rand::prelude::*;
    use std::collections::HashSet;
    let num_bucket_bits = 28;
    let mut hll = HLL::new(num_bucket_bits);
    let mut rng = rand::rng();
    let mut hashset = HashSet::<u64>::new();
    for _i in 0u64..1_000_000_000u64 {
        // let data = rng.random_range(1u64..1_000_000_000_000_000u64);
        let data = _i;
        // println!("{}", data);
        hll.read_data(data);
        // hashset.insert(data);
    }
    let results = hll.get_cardinality();
    println!("Finished Cardinality Estimatings with results = {} and real cardinality = {}!", results, hashset.len());
    // println!("{:#?}", hll.buckets());
    // for (index,i) in hll.buckets().iter().enumerate() {
    //     if i != &0 {
    //         println!("{}: {}", index, i);
    //         // let _ = io::stdout().flush();
    //     }
    // }
    // println!("hi?");
}


