mod vanilla;
mod universalhash;
mod benchmark;
mod hllpp;

use vlq::{ReadVlqExt, WriteVlqExt};

fn main() {
    println!("Hello, world!");
    // benchmark::vanilla::run_benchmark_speed();
    // benchmark::vanilla::run_benchmark_accuracy();
}
