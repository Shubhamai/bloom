use crate::bitvec::BitVec;

use pyo3::prelude::*;
use siphasher::sip::SipHasher24;
use std::hash::{Hash, Hasher};

// Things I did and how I came up with this approach
// Initially I implemented bloom filter and was using Rust `bitvec` library for initial testing

// I started testing whether there was a big difference in the ratio between 0s and 1s,
// Since we will storing 10url, the eventual ratio of 0s and 1s will be the same

// Then, I started working on the simplest approach, storing bits as string in file,
// Since linux has limit of reading at lest a byte for files
// Quickly I realized it will be 8x storage inefficient, for 10b urls, we will approach ~ 500GB

// I learned from bitvec on an approach where we can do bit manipulation to read/write individual bits on Vec<u8>
// on u8, so I started implemented it for files

const KEY: u64 = 42;

#[pyclass]
pub struct BloomFilter {
    bitarray: BitVec,
    n_hashes: u8,
    n_items: usize,
    mem_size: usize,
    hashers: [SipHasher24; 2],
}

#[pymethods]
impl BloomFilter {
    #[new]
    #[pyo3(signature = (n_items, error_rate, db_path = "db.bin"))]
    pub fn new(n_items: usize, error_rate: f64, db_path: &str) -> PyResult<Self> {
        let (n_hashes, mem_size) = Self::calculate_hashes_mem_size(error_rate);

        println!("No. of items: {}", n_items);
        println!("No. of hashes: {}", n_hashes);
        println!("bits/item: {}", mem_size);
        println!(
            "Space required: {:.4} MB",
            (mem_size as f32 * n_items as f32) / 8. / 1024. / 1024.
        );

        // The file will be create if not present
        let bitarray = BitVec::new(db_path.to_owned(), mem_size * n_items).unwrap();

        // NOTE: Make sure the hash fit the entire bitarray
        let hash1 = SipHasher24::new_with_keys(KEY + 1, KEY + 2);
        let hash2 = SipHasher24::new_with_keys(KEY + 3, KEY + 4);

        Ok(BloomFilter {
            bitarray,
            n_hashes,
            mem_size,
            n_items,
            hashers: [hash1, hash2],
        })
    }

    pub fn add(&mut self, url: &str) -> PyResult<()> {
        for i in 0..self.n_hashes {
            let index = self.get_hash(url, i.into()) % self.bitarray.len();
            self.bitarray.set(index)?
        }
        Ok(())
    }

    pub fn contains(&mut self, url: &str) -> PyResult<bool> {
        for i in 0..self.n_hashes {
            let index = self.get_hash(url, i.into()) % self.bitarray.len();
            if !self.bitarray.get(index)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// https://en.wikipedia.org/wiki/Bloom_filter#Optimal_number_of_hash_functions
    #[staticmethod]
    fn calculate_hashes_mem_size(error_rate: f64) -> (u8, usize) {
        let n_hashes: u8 = (-error_rate.log2()).ceil() as u8;
        let mem_size: usize = (-1.44 * error_rate.log2()).ceil() as usize;
        (n_hashes, mem_size)
    }

    // Wasn't sure which implementation to use (different seeds, prefixes, etc..) to calculate multiple hash, looked around what other were doing
    // and using double hashing seemed like a popular & well used option due to better uniform distribution
    // NOTE: Still should test against other methods like prefixes and see how much error rate changes
    fn get_hash(&self, key: &str, i: u32) -> usize {
        let hash1 = {
            let mut hasher = self.hashers[0].clone();
            key.hash(&mut hasher);
            hasher.finish()
        };

        let hash2 = {
            let mut hasher = self.hashers[1].clone();
            key.hash(&mut hasher);
            hasher.finish()
        };

        if i == 0 {
            hash1 as usize
        } else if i == 1 {
            hash2 as usize
        } else {
            // Use double hashing for additional hashes
            (hash1.wrapping_add(u64::from(i).wrapping_mul(hash2)) % self.bitarray.len() as u64)
                as usize
        }
    }
}

// Some crude calculations :)
// The theoretical performance of this: given 100m items, 48 bits/item and 34 hashes/item
// Seek speed in SSD ~ 0.1ms
// adding individual item will take about 0.1ms*34(hashes/item) = 3.4ms (300 items/sec)
// My PC (SSD) seems to be getting about 600 items/sec
// nvme are much faster (~0.01ms) and concurrent reading on M3 Pro was an order to magnitude faster.

// Improvements for incremental speedups:
// - Store the indices of zero in hashmap (in memory), this will help in ruling out urls that are not added faster
// -

// Improvements for order to magnitude speedups:
// The major bottleeneck is slow disk read/seek speed, due to the nature of bloom filter-
// We are randomlly accessing bits across the disk and have almost no chance of a cache hit.
// Since SSD and nvme allow concurrent read/write, that's very helpful, but
// The next system (assuming it's bloom filter and more performance is needed)
// will have to take advantage of disk page caching
// https://codecapsule.com/2014/02/12/coding-for-ssds-part-6-a-summary-what-every-programmer-should-know-about-solid-state-drives/
