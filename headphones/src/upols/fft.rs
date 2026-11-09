use std::sync::Arc;

use num::Complex;
use realfft::{ComplexToReal, FftNum, RealFftPlanner, RealToComplex};

pub struct RealToComplexHandler<T> {
    rtc: Arc<dyn RealToComplex<T>>,
    input: Box<[T]>,
    scratch: Box<[Complex<T>]>,
    output: Box<[Complex<T>]>,
}

impl<T: FftNum> RealToComplexHandler<T> {
    pub fn new(planner: &mut RealFftPlanner<T>, block_size: usize) -> Self {
        let rtc = planner.plan_fft_forward(block_size * 2);
        let input = rtc.make_input_vec().into_boxed_slice();
        let scratch = rtc.make_scratch_vec().into_boxed_slice();
        let output = rtc.make_output_vec().into_boxed_slice();
        Self {
            rtc,
            input,
            scratch,
            output,
        }
    }
    pub fn process(&mut self) {
        let _ = self
            .rtc
            .process_with_scratch(&mut self.input, &mut self.output, &mut self.scratch);
    }
    #[allow(unused)]
    pub fn input(&self) -> &[T] {
        &self.input
    }
    pub fn input_mut(&mut self) -> &mut [T] {
        &mut self.input
    }
    pub fn output(&self) -> &[Complex<T>] {
        &self.output
    }
    #[allow(unused)]
    pub fn output_mut(&mut self) -> &mut [Complex<T>] {
        &mut self.output
    }
}

pub struct ComplexToRealHandler<T> {
    ctr: Arc<dyn ComplexToReal<T>>,
    input: Box<[Complex<T>]>,
    scratch: Box<[Complex<T>]>,
    output: Box<[T]>,
}

impl<T: FftNum> ComplexToRealHandler<T> {
    pub fn new(planner: &mut RealFftPlanner<T>, block_size: usize) -> Self {
        let ctr = planner.plan_fft_inverse(block_size * 2);
        let input = ctr.make_input_vec().into_boxed_slice();
        let scratch = ctr.make_scratch_vec().into_boxed_slice();
        let output = ctr.make_output_vec().into_boxed_slice();
        Self {
            ctr,
            input,
            scratch,
            output,
        }
    }
    pub fn process(&mut self) {
        let _ = self
            .ctr
            .process_with_scratch(&mut self.input, &mut self.output, &mut self.scratch);
    }
    #[allow(unused)]
    pub fn input(&self) -> &[Complex<T>] {
        &self.input
    }
    pub fn input_mut(&mut self) -> &mut [Complex<T>] {
        &mut self.input
    }
    pub fn output(&self) -> &[T] {
        &self.output
    }
    pub fn output_mut(&mut self) -> &mut [T] {
        &mut self.output
    }
}
