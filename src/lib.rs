mod bitvec;
mod bloom_filter;

use bloom_filter::BloomFilter;
use pyo3::prelude::*;

#[pymodule]
fn bloom(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BloomFilter>()?;
    Ok(())
}
