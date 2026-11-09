use std::ops::{AddAssign, MulAssign};

use num::{cast::AsPrimitive, traits::ConstZero};
use realfft::{RealFftPlanner, num_complex::Complex};

mod audio_input;
mod fft;
mod filters;
mod freq_delay_line;

use {
    audio_input::AudioInput,
    fft::{ComplexToRealHandler, RealToComplexHandler},
    filters::Filters,
    freq_delay_line::FreqDelayLine,
};

pub use realfft::FftNum;

pub trait Value
where
    Self: FftNum + ConstZero + Copy + Default + MulAssign,
{
}
impl<T> Value for T where T: FftNum + ConstZero + Copy + Default + MulAssign {}

pub struct Upols<T: FftNum> {
    _planner: RealFftPlanner<T>,
    rtc: RealToComplexHandler<T>,
    ctr: ComplexToRealHandler<T>,
    audio_input: AudioInput<T>,
    filters: Filters<T>,
    freq_delay_line: FreqDelayLine<T>,
    block_count: usize,
    block_size: usize,
    normalization_factor: T,
}

impl<T> Upols<T>
where
    T: Value,
    usize: AsPrimitive<T>,
    Complex<T>: AddAssign,
{
    pub fn new(source_count: usize, filter_length: usize, block_size: usize) -> Self {
        let block_count = calculate_block_count(filter_length, block_size);
        let mut planner = RealFftPlanner::new();
        let rtc = RealToComplexHandler::new(&mut planner, block_size);
        let ctr = ComplexToRealHandler::new(&mut planner, block_size);
        let audio_input = AudioInput::new(source_count, block_size);
        let filters = Filters::new(block_size, block_count, source_count);
        let freq_delay_line = FreqDelayLine::new(block_size, block_count, source_count);
        let normalization_factor = T::one() / (block_size * 2).as_();
        Self {
            _planner: planner,
            rtc,
            ctr,
            audio_input,
            filters,
            freq_delay_line,
            block_count,
            block_size,
            normalization_factor,
        }
    }
    /// Sets the filter for the specified `source` and `channel`
    pub fn update_filter(&mut self, source: usize, channel: usize, filter: &[T]) {
        self.rtc.input_mut()[self.block_size..].fill(T::zero());

        let full_chunks = filter.chunks(self.block_size).take(self.block_count - 1);
        for (block, filter_input_chunk) in full_chunks.enumerate() {
            self.rtc.input_mut()[0..self.block_size].copy_from_slice(filter_input_chunk);

            self.rtc.process();

            self.filters
                .filter_block_mut(source, block, channel)
                .copy_from_slice(self.rtc.output());
        }

        let last_chunk_start = self.block_size * (self.block_count - 1);
        let last_chunk = &filter[last_chunk_start..];
        let zeroes = self.block_size - last_chunk.len();

        self.rtc.input_mut()[0..last_chunk.len()].copy_from_slice(last_chunk);
        self.rtc.input_mut()[last_chunk.len()..last_chunk.len() + zeroes].fill(T::zero());
    }
    /**
    Gets a mutable slice of the current audio input block for the
    specified `source`
    */
    pub fn audio_input_mut(&mut self, source: usize) -> &mut [T] {
        self.audio_input.current_mut(source)
    }
    /**
    Push the current audio input block to the frequency-domain delayline
    for the specified `source`
    */
    pub fn push_audio_input(&mut self, source: usize) {
        let previous_input = self.audio_input.previous(source);
        let current_input = self.audio_input.current(source);
        let (rtc_input_1, rtc_input_2) = self.rtc.input_mut().split_at_mut(self.block_size);

        rtc_input_1.copy_from_slice(previous_input);
        rtc_input_2.copy_from_slice(current_input);

        self.rtc.process();

        let fdl_block = self.freq_delay_line.get_block_mut(source, 0);
        let rtc_output = self.rtc.output();
        fdl_block.copy_from_slice(rtc_output);
    }
    /**
    Perform convolution for the current filters and audio input block
    for the specified `source` and `channel`
    */
    pub fn convolve(&mut self, source: usize, channel: usize) {
        // Fill accumulator with first block product:
        let fdl_block = self.freq_delay_line.get_block(source, 0).iter();
        let filter_block = self.filters.filter_block(source, 0, channel).iter();
        for ((filter, fdl), acc) in filter_block
            .zip(fdl_block)
            .zip(self.ctr.input_mut().iter_mut())
        {
            *acc = filter * fdl;
        }

        // Add the rest of the block products:
        for block in 1..self.block_count {
            let fdl_block = self.freq_delay_line.get_block(source, block).iter();
            let filter_block = self.filters.filter_block(source, block, channel).iter();

            for ((filter, fdl), acc) in filter_block
                .zip(fdl_block)
                .zip(self.ctr.input_mut().iter_mut())
            {
                *acc += filter * fdl;
            }
        }

        self.ctr.process();
        self.normalize_output();
    }
    /**
    This method readies the audio input for the next block.

    This should be called *after* convolution of a block.
    */
    pub fn advance_audio_input(&mut self) {
        self.audio_input.advance();
        self.freq_delay_line.advance();
    }
    /**
    Returns a slice of the usable, non-discarded output samples
    that resulted from the last convolution

    This slice is the same length as the block size.
    */
    pub fn output(&self) -> &[T] {
        &self.ctr.output()[self.block_size..]
    }
    fn normalize_output(&mut self) {
        for i in self.ctr.output_mut()[self.block_size..].iter_mut() {
            *i *= self.normalization_factor;
        }
    }
}

fn calculate_block_count(filter_length: usize, block_size: usize) -> usize {
    filter_length.div_ceil(block_size)
}

fn dimensional_index<const D: usize>(sizes: [usize; D], indexes: [usize; D]) -> usize {
    let mut sum = indexes[0];
    for (i, index) in indexes.iter().enumerate().take(D).skip(1) {
        let mut product = sizes[0];
        for size in sizes.iter().take(i).skip(1) {
            product *= size;
        }
        sum += index * product;
    }
    sum
}

#[test]
fn block_longer_than_filter() {
    let block_size = 128;
    let filter_length = 43;
    let block_count = calculate_block_count(filter_length, block_size);
    assert_eq!(block_count, 1);
}
