use num::Complex;

use super::dimensional_index;

pub struct FreqDelayLine<T> {
    buf: Box<[Complex<T>]>,
    position: usize,
    transform_size: usize,
    block_count: usize,
    source_count: usize,
}

impl<T: Default + Copy> FreqDelayLine<T> {
    pub fn new(block_size: usize, block_count: usize, source_count: usize) -> Self {
        let transform_size = block_size + 1;
        let buf_len = transform_size * block_count * source_count;
        let buf = vec![Complex::<T>::default(); buf_len].into_boxed_slice();
        Self {
            buf,
            position: 0,
            transform_size,
            block_count,
            source_count,
        }
    }
    pub fn get_block(&self, source: usize, block: usize) -> &[Complex<T>] {
        let start = self.block_start(source, block);
        let end = start + self.transform_size;
        &self.buf[start..end]
    }
    pub fn get_block_mut(&mut self, source: usize, block: usize) -> &mut [Complex<T>] {
        let start = self.block_start(source, block);
        let end = start + self.transform_size;
        &mut self.buf[start..end]
    }
    pub fn block_start(&self, source: usize, block: usize) -> usize {
        dimensional_index(
            [self.transform_size, self.block_count, self.source_count],
            [
                (self.position * self.transform_size),
                (self.block_count - 1) - block,
                source,
            ],
        ) % self.buf.len()
    }
    pub fn advance(&mut self) {
        self.position += 1
    }
}
