mod vanilla;
mod universalhash;
mod benchmark;
mod hllpp;

use vlq::{ReadVlqExt, WriteVlqExt};

fn main() {
    println!("Hello, world!");
    // benchmark::vanilla::run_benchmark_speed();
    // benchmark::vanilla::run_benchmark_accuracy();
    benchmark::hllpp::benchmarK_accuracy_hll();
    // benchmark::hllpp::benchmarK_accuracy_hllpp();
    // benchmark::hllpp::benchmarK_accuracy_hllpprh();
}
