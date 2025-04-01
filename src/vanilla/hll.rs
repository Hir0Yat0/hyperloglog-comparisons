use std::hash::Hash;
use crate::universalhash::*;

pub struct HLL {
    num_bucket_bits: usize,
    num_buckets: usize,
    bias_correction_value: f64,
    buckets: Vec<usize>, 
    hash_function: univesalhash::UniversalHashFunction,
}

impl HLL {
    pub fn new(num_bucket_bits: usize) -> Self {
        let num_buckets = 1 << num_bucket_bits;
        HLL { 
            num_bucket_bits: num_bucket_bits,
            num_buckets: num_buckets,
            bias_correction_value: Self::compute_bias_correction_value(num_buckets),
            buckets: vec![0,num_buckets],
            hash_function: univesalhash::UniversalHashFunctionsFamily::new(64).construct_new_hash_function_with_random_seeds(),
        }
    }

    pub fn read_stream(&mut self, input_stream: &mut Box<dyn Iterator<Item = u64>>){
        input_stream.map(|data| self.hash_function.hash64(data))
            .map(|hashed_data| (Self::get_bucket_idx(self.num_bucket_bits, hashed_data), Self::get_data_bits(self.num_bucket_bits, hashed_data)))
            .for_each(|(bucket_idx, data_bits)| {
                let leading_zeros = data_bits.leading_zeros() as usize;
                if leading_zeros < self.buckets.get(bucket_idx).unwrap_or(&0).clone() {
                    self.buckets[bucket_idx] = leading_zeros;
                }
            })
        ;
    }
    pub fn read_data(&mut self, data: u64){
        let hashed_data = self.hash(data);
        let (bucket_idx, data_bits) = (Self::get_bucket_idx(self.num_bucket_bits, hashed_data), Self::get_data_bits(self.num_bucket_bits, hashed_data));
        let leading_zeros = data_bits.leading_zeros() as usize;
        if leading_zeros < self.buckets.get(bucket_idx).unwrap_or(&0).clone() {
            self.buckets[bucket_idx] = leading_zeros;
        }
    }
    
    pub fn get_cardinality(&mut self) -> f64 {
        self.compute_estimates()
        
    }
    
    fn compute_bias_correction_value(num_buckets: usize) -> f64 {
        match num_buckets {
            // numbers precomputed from the paper
            16 => 0.673,
            32 => 0.697,
            64 => 0.709,
            // formula from hll paper
            _ => (0.7213 / (1f64 + (1.079 / (num_buckets as f64)))) * (num_buckets.pow(2) as f64)
        }
    }

    fn hash(&self, val: u64) -> u64 {
        self.hash_function.hash64(val)
    }

    fn compute_estimates(&mut self) -> f64 {
        let raw_estimates = self.bias_correction_value * Self::compute_mean_max_leading_zeroes(&self.buckets);
        Self::perform_correction(raw_estimates, &self.buckets)
    }

    fn compute_mean_max_leading_zeroes(buckets: &Vec<usize>) -> f64 {
        1.0f64  / (buckets.iter().map(|num_leading_zeroes| 1f64 / ((1 << num_leading_zeroes) as f64)).fold(0f64, |left,right| left + right))
    }

    fn perform_correction(raw_estimates: f64, buckets: &Vec<usize>) -> f64 {
        if Self::is_small_range(raw_estimates, buckets.len()) {
            Self::perform_small_range_correction(raw_estimates, buckets)
        }
        else if Self::is_large_range(raw_estimates) {
            Self::perform_large_range_correction(raw_estimates)
        }
        else {
            raw_estimates
        }
    }

    fn is_small_range(raw_estimates: f64, num_buckets: usize) -> bool {
        raw_estimates < 5f64/2f64 * (num_buckets as f64)
    }

    fn perform_small_range_correction(raw_estimates: f64, buckets: &Vec<usize>) -> f64 {
        let num_empty_buckets = Self::get_num_empty_buckets(buckets);
        if num_empty_buckets != 0 {
            Self::get_linear_counting_cardinality_estimates(buckets.len(), num_empty_buckets)
        }
        else {
            raw_estimates
        }
    }

    fn get_linear_counting_cardinality_estimates(num_buckets: usize, num_empty_buckets: usize) -> f64 {
        let num_buckets_f64 = num_buckets as f64;
        let num_empty_buckets_f64 = num_empty_buckets as f64;
        num_buckets_f64 * f64::ln(num_buckets_f64 / num_empty_buckets_f64)
    }
    

    fn get_num_empty_buckets(buckets: &Vec<usize>) -> usize {
        buckets.iter().map(|bucket| bucket == &0).map(|is_empty| if is_empty {0} else {1}).sum()
    }

    fn is_large_range(raw_estimates: f64) -> bool {
        // the original paper use 1/30 * 2^32 as threashold for 32 bits version
        // hll++ (64 bits) doesnt use large correction anymore
        // so i guess vanilla 64 bits would probably use 1/30 * 2^64...?
        raw_estimates > (1f64 / 30f64) * (2f64.powf(64.0))
    }

    fn perform_large_range_correction(raw_estimates: f64) -> f64 {
        - 2f64.powf(64.0) * f64::ln( 1f64 - (raw_estimates / (2f64.powf(32.0))))
    }

    fn get_bucket_idx(num_bucket_bits: usize, data: u64) -> usize {
        (data >> (64 - num_bucket_bits)) as usize
    }

    fn get_data_bits(num_bucket_bits: usize, data: u64) -> u64 {
        // get only the non buckets idx bits
        // idea: shift left by nums of bucket bits
        // then shift right back, now the buckets bits are all 0s
        // and the data bits stay the same
        (data << num_bucket_bits) >> num_bucket_bits
    }
}



