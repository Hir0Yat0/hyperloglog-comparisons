use rand::prelude::*;

pub struct UniversalHashFunctionsFamily {
    // family of universal hash functions constructed by multiply-shift method
    hash_results_num_bits: usize, 
    rng: ThreadRng,
}

impl UniversalHashFunctionsFamily {
    pub fn new(hash_results_num_bits: usize) -> Self {
        UniversalHashFunctionsFamily { hash_results_num_bits, rng: rand::rng() }
    }

    pub fn construct_new_hash_function(&self, seed_a: u128, seed_b: u128) -> UniversalHashFunction {
        UniversalHashFunction::new(self.hash_results_num_bits, seed_a, seed_b)
    }

    pub fn construct_new_hash_function_with_random_seeds(&mut self) -> UniversalHashFunction {
        let a = self.rng.random::<u128>();
        let b = self.rng.random::<u128>();
        self.construct_new_hash_function(a, b)
    }
}

pub struct UniversalHashFunction {
    hash_results_num_bits: usize,
    hash_function_parameter_seed_a: u128, 
    hash_function_parameter_seed_b: u128, 
}

impl UniversalHashFunction {
    const HASH_INPUTS_BIT_NUMS_2X: usize = 64 * 2;

    pub fn new(hash_results_num_bits: usize, hash_function_parameter_a: u128, hash_function_parameter_b: u128, ) -> Self {
        UniversalHashFunction {hash_results_num_bits , hash_function_parameter_seed_a: hash_function_parameter_a, hash_function_parameter_seed_b: hash_function_parameter_b, }
    }

    pub fn hash(&self, val: u64) -> u128 {
        self.hash128(val)
    }

    pub fn hash128(&self, val: u64) -> u128 {
        let val_u128 = val as u128;
        (self.hash_function_parameter_seed_a * val_u128 + self.hash_function_parameter_seed_b) >> (Self::HASH_INPUTS_BIT_NUMS_2X - self.hash_results_num_bits)
    }

    pub fn hash64(&self, val: u64) -> u64 {
        self.hash128(val) as u64
    }

    pub fn hash32(&self, val: u64) -> u32 {
        self.hash64(val) as u32
    }
}
