use super::dimensional_index;

pub struct AudioInput<T> {
    buf: Box<[T]>,
    position: bool,
    block_size: usize,
    source_count: usize,
}

impl<T: Copy + Default> AudioInput<T> {
    fn block_start(&self, source: usize, position: bool) -> usize {
        dimensional_index(
            [self.block_size * 2, self.source_count],
            [usize::from(position) * self.block_size, source],
        )
    }
    fn previous_start(&self, source: usize) -> usize {
        self.block_start(source, self.position)
    }
    fn current_start(&self, source: usize) -> usize {
        self.block_start(source, !self.position)
    }
    pub fn new(source_count: usize, block_size: usize) -> Self {
        let buf_len = source_count * (block_size * 2);
        let buf = vec![T::default(); buf_len].into_boxed_slice();
        Self {
            buf,
            position: false,
            block_size,
            source_count,
        }
    }
    pub fn current(&self, source: usize) -> &[T] {
        let start = self.current_start(source);
        let end = start + self.block_size;
        &self.buf[start..end]
    }
    pub fn current_mut(&mut self, source: usize) -> &mut [T] {
        let start = self.current_start(source);
        let end = start + self.block_size;
        &mut self.buf[start..end]
    }
    pub fn previous(&self, source: usize) -> &[T] {
        let start = self.previous_start(source);
        let end = start + self.block_size;
        &self.buf[start..end]
    }
    pub fn advance(&mut self) {
        self.position = !self.position;
    }
}
