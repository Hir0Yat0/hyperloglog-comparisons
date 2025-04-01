use rand::rngs::ThreadRng;

use crate::vanilla::hll::*;

fn run_benchmark_speed(){
    use std::time::{Duration, Instant};
    use rand::prelude::*;
    let num_bucket_bits = 12;
    let mut hll = HLL::new(num_bucket_bits);
    let mut rng: ThreadRng = rand::rng();
    let time_start_1 = Instant::now();
    // let input_stream = {
        
    //     (0u64..1_000_000_000u64).map(|_i| rng.next_u64()).into_iter().map(|_i| rng.next_u64())
    // };
    // hll.read_stream(&Box::new(input_stream));
    for _i in 0u64..1_000_000_000u64 {
        hll.read_data(rng.random());
    }
    let time_start_1_elapsed_1 = time_start_1.elapsed();
    println!("Finished Reading Stream and Counting in {} secs", time_start_1_elapsed_1.as_secs_f64());
    let time_start_2 = Instant::now();
    let results = hll.get_cardinality();
    let time_start_2_elapsed_1 = time_start_2.elapsed();
    println!("Finished Cardinality Estimatings in {} secs with results = {}!", time_start_2_elapsed_1.as_secs_f64(), results);
}

fn run_benchmark_accuracy() {
    unimplemented!()
}


