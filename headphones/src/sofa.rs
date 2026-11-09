use std::path::Path;

use sofar::reader::{Filter, OpenOptions, Sofar};

/// Reads filters from a SOFA file
pub struct Sofa {
    sofa: Sofar,
    filters: Filter,
}

impl Sofa {
    /// Loads a SOFA file from a path
    pub fn new_from_file<P>(path: P, sample_rate: u32) -> Result<Self, sofar::reader::Error>
    where
        P: AsRef<Path>,
    {
        let sofa = OpenOptions::new()
            .sample_rate(sample_rate as f32)
            .open(path)?;
        let filter_len = sofa.filter_len();
        Ok(Self {
            sofa,
            filters: Filter::new(filter_len),
        })
    }
    /// Loads a SOFA file from bytes
    pub fn new_from_data<B>(bytes: B, sample_rate: u32) -> Result<Self, sofar::reader::Error>
    where
        B: AsRef<[u8]>,
    {
        let sofa = OpenOptions::new()
            .sample_rate(sample_rate as f32)
            .open_data(bytes)?;
        let filter_len = sofa.filter_len();
        Ok(Self {
            sofa,
            filters: Filter::new(filter_len),
        })
    }
    /// Gets the left and right filters for the given `position`
    pub(crate) fn filters(&mut self, position: [f32; 3]) -> [&[f32]; 2] {
        let [x, y, z] = position;
        self.sofa.filter(x, y, z, &mut self.filters);
        [&self.filters.left, &self.filters.right]
    }
    pub(crate) fn filter_length(&self) -> usize {
        self.sofa.filter_len()
    }
}
