mod vanilla;
mod universalhash;
mod benchmark;

fn main() {
    println!("Hello, world!");
    benchmark::vanilla::run_benchmark_speed();
}
