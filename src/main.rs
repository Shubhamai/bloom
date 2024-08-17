mod bitvec;
mod bloom_filter;

use std::{
    thread,
    time::{self, Instant},
};

use bloom_filter::BloomFilter;
use rand::{distributions::Alphanumeric, Rng};

fn main() {
    let n_items: usize = 100_000_000;
    let error_rate = 1e-10;
    let db_path = "testing.bin";
    let items_to_read = 10_000;

    let mut filter = BloomFilter::new(n_items, error_rate, db_path).unwrap();

    let random_strings = get_random_urls(items_to_read);

    // Sequential reads
    // let sequential_time = Instant::now();
    // random_strings.iter().for_each(|s| {
    //     filter.contains(s).unwrap();
    // });
    // println!(
    //     "Sequential read took around {:?} for {} items",
    //     sequential_time.elapsed(),
    //     items_to_read
    // );

    // Concurrent reads (only try if db is already created)
    let concurrent_time = Instant::now();
    thread::scope(|scope| {
        for chunk in random_strings.as_slice().chunks(random_strings.len() / 20) {
            scope.spawn(move || {
                let mut filter = BloomFilter::new(n_items, error_rate, db_path).unwrap();

                chunk.iter().for_each(|s| {
                    filter.contains(s).unwrap();
                })
            });
        }
    });
    println!(
        "Concurrent read took around {:?} for {} items",
        concurrent_time.elapsed(),
        items_to_read
    );
}

fn get_random_urls(n_urls: usize) -> Vec<String> {
    let mut all_strings = vec![];
    for _ in 0..n_urls {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        all_strings.push(s);
    }
    all_strings
}
