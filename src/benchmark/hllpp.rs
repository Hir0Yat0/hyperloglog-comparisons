use crate::hllpp::*;
use crate::vanilla;

pub fn benchmarK_accuracy_hll() {
    for cardinality in 0u32..9 {
        // println!("c{}",cardinality);
        let mut relative_errors = vec![0f64;100];
        for run_i in 0..100 {
            // println!("r{}",run_i);
            let mut hllpp = vanilla::hll::HLL::new(28);
            for data in 0..(10u64.pow(cardinality)) {
                hllpp.read_data(data);
            }
            let estimates = hllpp.get_cardinality();
            let relative_error = compute_relative_error(estimates, (10u64.pow(cardinality)) as f64);
            relative_errors[run_i] = relative_error;
        }
        let median_relative_error = median(&relative_errors);
        println!("{}, ", median_relative_error);
    }
    println!("");
}

pub fn benchmarK_accuracy_hllpp() {
    for cardinality in 0u32..9 {
        // println!("c{}",cardinality);
        let mut relative_errors = vec![0f64;100];
        for run_i in 0..100 {
            // println!("r{}",run_i);
            let mut hllpp = hllpp::HLLPP::<14,25>::new();
            for data in 0..(10u64.pow(cardinality)) {
                hllpp.read_data(data);
            }
            let estimates = hllpp.compute_estimates();
            let relative_error = compute_relative_error(estimates, (10u64.pow(cardinality)) as f64);
            relative_errors[run_i] = relative_error;
        }
        let median_relative_error = median(&relative_errors);
        println!("{}, ", median_relative_error);
    }
    println!("");
}

pub fn benchmarK_accuracy_hllpprh() {
    for cardinality in 0u32..9 {
        let mut relative_errors = vec![0f64;100];
        for run_i in 0..100 {
            let mut hllpprh = hllpprh::HLLPPRH::<14,25>::new();
            for data in 0..(10u64.pow(cardinality)) {
                hllpprh.read_data(data);
            }
            let estimates = hllpprh.compute_estimates();
            let relative_error = compute_relative_error(estimates, (10u64.pow(cardinality)) as f64);
            relative_errors[run_i] = relative_error;
        }
        let median_relative_error = median(&relative_errors);
        println!("{}, ", median_relative_error);
    }
    println!("");
}

fn compute_relative_error(approx_val: f64, true_val: f64) -> f64 {
    f64::abs(approx_val - true_val) / f64::abs(true_val)
}

fn median(vec : &Vec<f64>) -> f64 {
    let mut count = 0;
    let mut current_max = vec[0];
    for &i in vec {
        if current_max >= i {
            count += 1;
            current_max = i;
        }
        if count >= vec.len() / 2 {
            return current_max;
        }
    }
    return *vec.last().unwrap();
}

