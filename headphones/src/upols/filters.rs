use num::Complex;

use super::dimensional_index;

pub struct Filters<T> {
    buf: Box<[Complex<T>]>,
    block_size: usize,
    block_count: usize,
    source_count: usize,
}

impl<T: Default + Copy> Filters<T> {
    pub fn new(block_size: usize, block_count: usize, source_count: usize) -> Self {
        let len = (((block_size + 1) * block_count) * source_count) * 2;
        let buf = vec![Complex::<T>::default(); len].into_boxed_slice();
        Self {
            buf,
            block_size,
            block_count,
            source_count,
        }
    }

    pub fn filter_block(&self, source: usize, block: usize, channel: usize) -> &[Complex<T>] {
        let start = dimensional_index(
            [self.block_size + 1, self.block_count, 2, self.source_count],
            [0, block, channel, source],
        );
        let end = start + (self.block_size + 1);
        &self.buf[start..end]
    }
    pub fn filter_block_mut(
        &mut self,
        source: usize,
        block: usize,
        channel: usize,
    ) -> &mut [Complex<T>] {
        let start = dimensional_index(
            [self.block_size + 1, self.block_count, 2, self.source_count],
            [0, block, channel, source],
        );
        let end = start + (self.block_size + 1);
        &mut self.buf[start..end]
    }
}
