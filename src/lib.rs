extern crate bitarray;
extern crate fnv;
extern crate murmur3;

use std::hash::Hasher;
use fnv::FnvHasher;
use std::io::Cursor;

use bitarray::BitArray;

pub struct BloomFilter {
    pub filter: BitArray,
    pub size: u64,
    hash_funcs: Vec<HashFn>,
    total_flipped: i64,
}

pub type HashFn = fn(&str) -> u64;

fn hash_fnv1a(bytes: &str) -> u64 {
    let mut hasher = FnvHasher::default();
    hasher.write(bytes.as_bytes());
    hasher.finish()
}

fn hash_murmur3(bytes: &str) -> u64 {
    let mut out: [u8; 16] = [0; 16];
    murmur3::murmur3_x64_128(&mut Cursor::new(bytes), 0, &mut out);
    let mut hash: u64 = 0;
    for byte in out.iter() {
        hash = hash ^ (*byte as u64);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl BloomFilter {
    pub fn new(filter_size: u64) -> BloomFilter {
        let mut hash_funcs: Vec<HashFn> = Vec::new();
        hash_funcs.push(hash_fnv1a);
        hash_funcs.push(hash_murmur3);
        BloomFilter {
            filter: BitArray::new(),
            size: BitArray::new().size(),
            hash_funcs: hash_funcs,
            total_flipped: 0,
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        // Put the candidate string through each of the hash functions, and for each
        // index returned get the value at that index in the bloom filter, and put
        // those values into the results slice.
        let mut results: Vec<bool> = vec![false; self.hash_funcs.len()];
        for i in 0..self.hash_funcs.len() as usize {
            let idx = self.calc_index(key, self.hash_funcs[i]);
            let val = self.filter.get(idx);
            results[i] = val;
        }
        let mut all_true = true;
        // Iterate over the switch values retrieved from the bloom filter.
        for j in results {
            // If the switch values retrieved are all true we'll end up returning true.
            all_true = all_true && j;
        }
        all_true
    }

    pub fn add(&mut self, s: &str) {
        // Iterate over the list of hash functions, using each to reduce the string
        // to a single index in the filter, which is then flipped on.
        for j in &self.hash_funcs {
            let idx = self.calc_index(s, *j);
            self.filter.set(idx, true);
            self.total_flipped += 1;
        }
    }

    pub fn calc_index(&self, s: &str, hash_fn: HashFn) -> u64 {
        let hash = hash_fn(s);
        let rem = hash % self.size;
        rem
    }
}


#[cfg(test)]
mod tests {
    use super::BloomFilter;  // Alternatively: use super::*;

    #[test]
    fn exists() {

        // Strings to add to the filter as sample existing data.
        let load_strings: [&str; 7] =
            ["cat", "dog", "mate", "frog", "moose", "el capitan", "spruce goose"];

        // Test struct to hold each test case.
        struct TestCase {
            input: &'static str,
            expected: bool,
        }

        // Build a list of test cases.
        let testcases: [TestCase; 6] = [TestCase {
                                            input: "klingon",
                                            expected: false,
                                        },
                                        TestCase {
                                            input: "frog",
                                            expected: true,
                                        },
                                        TestCase {
                                            input: "donkey",
                                            expected: false,
                                        },
                                        TestCase {
                                            input: "tame",
                                            expected: false,
                                        },
                                        TestCase {
                                            input: "spruce goose",
                                            expected: true,
                                        },
                                        TestCase {
                                            input: "light speed",
                                            expected: false,
                                        }];

        // Create a new BloomFilter and add the sample data.
        let mut b = BloomFilter::new(100);
        for j in 0..load_strings.len() {
            b.add(load_strings[j]);
        }

        // Test each case.
        for i in 0..testcases.len() {
            let actual = b.exists(testcases[i].input);
            println!("Checking key: {} ; returned: {}",
                     testcases[i].input,
                     actual);
            assert_eq!(actual, testcases[i].expected);
        }
    }
}
