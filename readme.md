## Try it out

Install [Rust and Cargo](https://www.rust-lang.org/tools/install)

To test python bindings

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin
maturin develop
python example.py
```

To test concurrent reads in rust

```bash
cargo run --release
```




Things I did and how I came up with this approach

- Initially I implemented bloom filter and was using Rust `bitvec` library for initial testing


- I started testing whether there was a big difference in the ratio between 0s and 1s,
- Since we will be storing 10b urls, the eventual ratio of 0s and 1s will be the same


- Then, I started working on the simplest approach, storing bits as string in file,
- Since linux has limit of reading at least a byte for files
- Quickly I realized it will be 8x storage inefficient, for 10b urls, we will approach ~ 500GB


- I learned from bitvec on an approach where we can do bit manipulation to read/write individual bits on `Vec<u8>`, so I started implemented it for files

---

Some crude calculations :)

- The best performance of this: given 100m items, 48 bits/item and 34 hashes/item
- Seek speed in SSD ~ 0.1ms
- adding individual item will take about 0.1ms\*34(hashes/item) = 3.4ms (300 items/sec)
- My PC (SSD) seems to be getting about 600 items/sec
- nvme are much faster (~0.01ms) and initial testing of concurrent reading on M3 Pro was an order to magnitude faster.

- Improvements for incremental speedups:

  - Store the indices of zero in hashmap (in memory), this will help in ruling out urls that are not added faster
  - Concurrent read/write

---

Improvements for order to magnitude speedups:

- The major bottleeneck is slow disk read/seek speed, due to the nature of bloom filter-
- We are randomlly accessing bits across the disk and have almost no chance of a cache hit.

- Since SSD and nvme allow concurrent read/write, that's very helpful, but

- The next system (assuming it's bloom filter and more performance is needed)
- will have to take advantage of disk page caching

- https://codecapsule.com/2014/02/12/coding-for-ssds-part-6-a-summary-what-every-programmer-should-know-about-solid-state-drives/
