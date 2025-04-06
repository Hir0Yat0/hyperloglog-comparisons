use crate::universalhash::univesalhash::*;
use std::collections::HashMap;
use vint64;
use vlq::{ReadVlqExt, WriteVlqExt};

/* use encodehashtype u64 if sparseprecision is > 25 (25 + 6 + 1 = 32) */
type EncodeHashType = u32;
// type EncodeHashType = u64;

enum Format {
    NORMAL,
    SPARSE,
}   

pub struct HLLPPRH<const Precision: usize, const SparsePrecision: usize> {
    format: Format,
    tmp_set: HashMap<usize,EncodedHash>,
    sparse_list: VariableLengthU64Array,
    hasher: Vec<UniversalHashFunction>,
    buckets: Vec<u8>,
}

impl<const Precision: usize, const SparsePrecision: usize> HLLPPRH<Precision, SparsePrecision> {
    const NUM_BUCKETS: usize = 1 << Precision;
    const NUM_BUCKETS_F64: f64 = Self::NUM_BUCKETS as f64;
    const NUM_BUCKETS_SPARSE: usize = 1 << SparsePrecision;
    const NUM_BUCKETS_SPARSE_F64: f64 = Self::NUM_BUCKETS_SPARSE as f64;
    const BIAS_CORRECTION_VALUE: f64 = (match Self::NUM_BUCKETS {
        16 => 0.673f64,
        32 => 0.697f64,
        64 => 0.709f64,
        _ => 0.7213f64 / (1f64 + (1.079f64 / Self::NUM_BUCKETS_F64)),
    }) * (Self::NUM_BUCKETS_F64 * Self::NUM_BUCKETS_F64);

    pub fn new() -> Self {
        Self {
            format: Format::SPARSE,
            tmp_set: HashMap::new(),
            sparse_list: VariableLengthU64Array::new(),
            hasher: Self::gen_hash_functions(),
            buckets: vec![],
        }
    }

    fn gen_hash_functions () -> Vec<UniversalHashFunction> {
        let mut h = Vec::new();
        for i in 0..Self::NUM_BUCKETS {
            h[i] = UniversalHashFunctionsFamily::new(64).construct_new_hash_function_with_random_seeds()
        }
        h
    }

    pub fn read_data(&mut self, data: u64) {
        let hashed_data = self.hasher[(data % (self.hasher.len() as u64)) as usize].hash64(data);
        match self.format {
            Format::NORMAL => {
                let (bucket_idx, data_bits) = (Self::get_bucket_idx(Precision, hashed_data), Self::get_data_bits(Precision, hashed_data));
                let leading_zeros = data_bits.leading_zeros() as u8;
                if leading_zeros > self.buckets[bucket_idx] {
                    self.buckets[bucket_idx] = leading_zeros;
                }
            },
            Format::SPARSE => {
                let (sparse_bucket_idx, encoded) = Self::encode(hashed_data);
                self.tmp_set.insert(sparse_bucket_idx, encoded);
                if self.tmp_set.len() > Self::NUM_BUCKETS * 6 {
                    self.format = Format::NORMAL;
                    self.to_normal();
                }
            },
        }
    }

    pub fn compute_estimates(&self) -> f64 {
        match self.format {
            Format::SPARSE => {
                Self::linear_counting(Self::NUM_BUCKETS_SPARSE, self.tmp_set.len())
            },
            Format::NORMAL => {
                let mut raw_estimates = self.compute_mean_leading_zeros();
                if raw_estimates <= 5f64 * Self::NUM_BUCKETS_F64 {
                    raw_estimates = raw_estimates - Self::estimate_bias(raw_estimates);
                }
                let num_empty_buckets = self.buckets.iter().filter(|&&i| i == 0).count();
                let linear_counting_estimates = if num_empty_buckets != 0 {
                    Self::linear_counting(Self::NUM_BUCKETS, num_empty_buckets)
                }
                else {
                    raw_estimates
                };
                if linear_counting_estimates <= Self::threashold() {
                    linear_counting_estimates
                }
                else {
                    raw_estimates
                }
            },
        }
    }

    fn compute_mean_leading_zeros(&self) -> f64 {
        let mut total: f64 = 0f64;
        for num_leading_zeros in &self.buckets {
            total += 1f64 / ((1 << num_leading_zeros) as f64)
        }
        1f64 / total
    }

    fn get_bucket_idx(num_bucket_bits: usize, data: u64) -> usize {
        // println!("data = {:#016x}",data);
        // dbg!((data >> (64 - num_bucket_bits)) as usize)
        (data >> (64 - num_bucket_bits)) as usize
    }

    fn get_data_bits(num_bucket_bits: usize, data: u64) -> u64 {
        // get only the non buckets idx bits
        // idea: shift left by nums of bucket bits
        // then shift right back, now the buckets bits are all 0s
        // and the data bits stay the same
        (data << num_bucket_bits) >> num_bucket_bits
    }

    fn linear_counting(num_bucket: usize, num_empty_buckets: usize) -> f64 {
        let num_bucket_f64 = num_bucket as f64;
        let num_empty_bucket_f64 = num_empty_buckets as f64;
        num_bucket_f64 * f64::ln(num_bucket_f64 / num_empty_bucket_f64)
    }

    fn threashold() -> f64 {
        (match Precision {
            4 => 10,
            5 => 20,
            6 => 40,
            7 => 80,
            8 => 220,
            9 => 400,
            10 => 900,
            11 => 1800,
            12 => 3100,
            13 => 6500,
            14 => 11500,
            15 => 20000,
            16 => 50000,
            17 => 120000,
            18 => 350000,
            _ => 500000, // some random number for else case, with threashold for ~19
        }) as f64
    }

    fn estimate_bias(raw_estimates: f64) -> f64 {
        let raw_estimates_data: [f64; 201] = match Precision {
            _ => unimplemented!(),
            14 => {
                [ 11817.475, 12015.0046, 12215.3792, 12417.7504, 12623.1814, 12830.0086, 13040.0072, 13252.503, 13466.178, 13683.2738, 13902.0344, 14123.9798, 14347.394, 14573.7784, 14802.6894, 15033.6824, 15266.9134, 15502.8624, 15741.4944, 15980.7956, 16223.8916, 16468.6316, 16715.733, 16965.5726, 17217.204, 17470.666, 17727.8516, 17986.7886, 18247.6902, 18510.9632, 18775.304, 19044.7486, 19314.4408, 19587.202, 19862.2576, 20135.924, 20417.0324, 20697.9788, 20979.6112, 21265.0274, 21550.723, 21841.6906, 22132.162, 22428.1406, 22722.127, 23020.5606, 23319.7394, 23620.4014, 23925.2728, 24226.9224, 24535.581, 24845.505, 25155.9618, 25470.3828, 25785.9702, 26103.7764, 26420.4132, 26742.0186, 27062.8852, 27388.415, 27714.6024, 28042.296, 28365.4494, 28701.1526, 29031.8008, 29364.2156, 29704.497, 30037.1458, 30380.111, 30723.8168, 31059.5114, 31404.9498, 31751.6752, 32095.2686, 32444.7792, 32794.767, 33145.204, 33498.4226, 33847.6502, 34209.006, 34560.849, 34919.4838, 35274.9778, 35635.1322, 35996.3266, 36359.1394, 36722.8266, 37082.8516, 37447.7354, 37815.9606, 38191.0692, 38559.4106, 38924.8112, 39294.6726, 39663.973, 40042.261, 40416.2036, 40779.2036, 41161.6436, 41540.9014, 41921.1998, 42294.7698, 42678.5264, 43061.3464, 43432.375, 43818.432, 44198.6598, 44583.0138, 44970.4794, 45353.924, 45729.858, 46118.2224, 46511.5724, 46900.7386, 47280.6964, 47668.1472, 48055.6796, 48446.9436, 48838.7146, 49217.7296, 49613.7796, 50010.7508, 50410.0208, 50793.7886, 51190.2456, 51583.1882, 51971.0796, 52376.5338, 52763.319, 53165.5534, 53556.5594, 53948.2702, 54346.352, 54748.7914, 55138.577, 55543.4824, 55941.1748, 56333.7746, 56745.1552, 57142.7944, 57545.2236, 57935.9956, 58348.5268, 58737.5474, 59158.5962, 59542.6896, 59958.8004, 60349.3788, 60755.0212, 61147.6144, 61548.194, 61946.0696, 62348.6042, 62763.603, 63162.781, 63560.635, 63974.3482, 64366.4908, 64771.5876, 65176.7346, 65597.3916, 65995.915, 66394.0384, 66822.9396, 67203.6336, 67612.2032, 68019.0078, 68420.0388, 68821.22, 69235.8388, 69640.0724, 70055.155, 70466.357, 70863.4266, 71276.2482, 71677.0306, 72080.2006, 72493.0214, 72893.5952, 73314.5856, 73714.9852, 74125.3022, 74521.2122, 74933.6814, 75341.5904, 75743.0244, 76166.0278, 76572.1322, 76973.1028, 77381.6284, 77800.6092, 78189.328, 78607.0962, 79012.2508, 79407.8358, 79825.725, 80238.701, 80646.891, 81035.6436, 81460.0448, 81876.3884, ]
            }
        };
        let bias_data: [f64; 201] = match Precision {
            _ => unimplemented!(),
            14 => {
                [ 11816.475, 11605.0046, 11395.3792, 11188.7504, 10984.1814, 10782.0086, 10582.0072, 10384.503, 10189.178, 9996.2738, 9806.0344, 9617.9798, 9431.394, 9248.7784, 9067.6894, 8889.6824, 8712.9134, 8538.8624, 8368.4944, 8197.7956, 8031.8916, 7866.6316, 7703.733, 7544.5726, 7386.204, 7230.666, 7077.8516, 6926.7886, 6778.6902, 6631.9632, 6487.304, 6346.7486, 6206.4408, 6070.202, 5935.2576, 5799.924, 5671.0324, 5541.9788, 5414.6112, 5290.0274, 5166.723, 5047.6906, 4929.162, 4815.1406, 4699.127, 4588.5606, 4477.7394, 4369.4014, 4264.2728, 4155.9224, 4055.581, 3955.505, 3856.9618, 3761.3828, 3666.9702, 3575.7764, 3482.4132, 3395.0186, 3305.8852, 3221.415, 3138.6024, 3056.296, 2970.4494, 2896.1526, 2816.8008, 2740.2156, 2670.497, 2594.1458, 2527.111, 2460.8168, 2387.5114, 2322.9498, 2260.6752, 2194.2686, 2133.7792, 2074.767, 2015.204, 1959.4226, 1898.6502, 1850.006, 1792.849, 1741.4838, 1687.9778, 1638.1322, 1589.3266, 1543.1394, 1496.8266, 1447.8516, 1402.7354, 1361.9606, 1327.0692, 1285.4106, 1241.8112, 1201.6726, 1161.973, 1130.261, 1094.2036, 1048.2036, 1020.6436, 990.901400000002, 961.199800000002, 924.769800000002, 899.526400000002, 872.346400000002, 834.375, 810.432000000001, 780.659800000001, 756.013800000001, 733.479399999997, 707.923999999999, 673.858, 652.222399999999, 636.572399999997, 615.738599999997, 586.696400000001, 564.147199999999, 541.679600000003, 523.943599999999, 505.714599999999, 475.729599999999, 461.779600000002, 449.750800000002, 439.020799999998, 412.7886, 400.245600000002, 383.188199999997, 362.079599999997, 357.533799999997, 334.319000000003, 327.553399999997, 308.559399999998, 291.270199999999, 279.351999999999, 271.791400000002, 252.576999999997, 247.482400000001, 236.174800000001, 218.774599999997, 220.155200000001, 208.794399999999, 201.223599999998, 182.995600000002, 185.5268, 164.547400000003, 176.5962, 150.689599999998, 157.8004, 138.378799999999, 134.021200000003, 117.614399999999, 108.194000000003, 97.0696000000025, 89.6042000000016, 95.6030000000028, 84.7810000000027, 72.635000000002, 77.3482000000004, 59.4907999999996, 55.5875999999989, 50.7346000000034, 61.3916000000027, 50.9149999999936, 39.0384000000049, 58.9395999999979, 29.633600000001, 28.2032000000036, 26.0078000000067, 17.0387999999948, 9.22000000000116, 13.8387999999977, 8.07240000000456, 14.1549999999988, 15.3570000000036, 3.42660000000615, 6.24820000000182, -2.96940000000177, -8.79940000000352, -5.97860000000219, -14.4048000000039, -3.4143999999942, -13.0148000000045, -11.6977999999945, -25.7878000000055, -22.3185999999987, -24.409599999999, -31.9756000000052, -18.9722000000038, -22.8678000000073, -30.8972000000067, -32.3715999999986, -22.3907999999938, -43.6720000000059, -35.9038, -39.7492000000057, -54.1641999999993, -45.2749999999942, -42.2989999999991, -44.1089999999967, -64.3564000000042, -49.9551999999967, -42.6116000000038, ]
            }
        };
        if &raw_estimates > raw_estimates_data.last().unwrap() {
            *bias_data.last().unwrap()
        }
        else {
            for (idx, &i) in raw_estimates_data.iter().enumerate() {
                if raw_estimates > i {
                    return (bias_data[idx] - bias_data[idx + 1]) / 2f64
                }
            }
            // *bias_data.last().unwrap()
            unreachable!()
        }
    }

    // fn encode_hash(hashed_data: u64) -> EncodeHashType {
    //     // encode hashed_data as integer
    //     let (sparse_bucket_idx, sparse_data_bits) = (Self::get_bucket_idx(SparsePrecision, hashed_data), Self::get_data_bits(SparsePrecision, hashed_data));
    //     let sparse_bucket_idx_u32 = sparse_bucket_idx as u32;
    //     let bucket_idx_between_sparsed_and_normal = (hashed_data << Precision) >> (Precision + (64 - SparsePrecision));
    //     if bucket_idx_between_sparsed_and_normal == 0 {
    //         let leading_zeros = sparse_data_bits.leading_zeros();
    //         (sparse_bucket_idx_u32 << 7) | (leading_zeros << 1) | 1
    //     }
    //     else {
    //         sparse_bucket_idx_u32 << 1
    //     }
    // }

    // fn decode_hash() {
        
    // }

    // fn get_index(encoded_hashed_value: EncodeHashType) {
    //     let num_leading_zeros = if encoded_hashed_value & (1 as EncodeHashType) == (1 as EncodeHashType) {

    //     }
    //     else {

    //     };
    // }

    fn encode (hashed_data: u64) -> (usize, EncodedHash) {
        let (sparse_bucket_idx, sparse_data_bits) = (Self::get_bucket_idx(SparsePrecision, hashed_data), Self::get_data_bits(SparsePrecision, hashed_data));
        let (normal_bucket_idx, data_bits) = (Self::get_bucket_idx(Precision, hashed_data), Self::get_data_bits(Precision, hashed_data));
        let sparse_leading_zeros = sparse_data_bits.leading_zeros() as u8;
        let normal_leading_zeros = data_bits.leading_zeros() as u8;
        (sparse_bucket_idx, EncodedHash {sparse_leading_zeros, normal_leading_zeros, normal_bucket_idx})
    }
    
    fn decode(encoded: EncodedHash) -> (usize, u8) {
        (encoded.normal_bucket_idx, encoded.normal_leading_zeros)
    }

    fn to_normal(&mut self) {
        self.buckets = vec![0;Self::NUM_BUCKETS];
        for (_sparse_bucket_idx, encoded) in self.tmp_set.drain() {
            self.buckets[encoded.normal_bucket_idx] = encoded.normal_leading_zeros;
        }
        self.tmp_set.shrink_to_fit();
    }

}

struct VariableLengthU64Array {
    data: std::io::Cursor<Vec<u8>>,
}

impl VariableLengthU64Array {
    fn new() -> Self {
        let data: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(vec![]);
        Self {
            data
        }
    }

    fn write(&mut self, data: u64) -> usize {
        let prev_cursor_pos = self.data.position();
        self.data.write_vlq(data).unwrap();
        (self.data.position() - prev_cursor_pos) as usize
    }

    fn read(&mut self) -> u64 {
        self.data.read_vlq().unwrap()
    }
}

#[derive(Hash)]
struct EncodedHash{
    sparse_leading_zeros: u8,
    normal_leading_zeros: u8,
    normal_bucket_idx: usize,
}
