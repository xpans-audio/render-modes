/*!
Implementation of xpans' Stereo rendering mode
*/
#![no_std]
use core::marker::PhantomData;
pub mod pan_law;
use nalgebra::{SimdRealField, Vector3};
use num::{Float, FromPrimitive};
use pan_law::PanLaw;
use xpans::{Extent, Position};
use xpans_common_lr::{FlipSign, gain};
use xpans_render::mode::*;

/**
Interprets audio sources based on their position within the scene.

Positional stereo is more physically accurate in typical stereo speaker
setups as opposed to headphones.
*/
#[derive(Default, Clone, Copy)]
pub struct Positional<T> {
    phantom_data: PhantomData<T>,
}
impl<T: Default> InterpretationInfo for Positional<T> {
    type Interpretation = T;

    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<S, I, T> InterpretSources<S, I> for Positional<T>
where
    T: SimdRealField + Copy + Default,
    I: Interpretations<T>,
    S: Sources,
    S::Source: Position<T> + Extent<T>,
{
    fn interpret_sources(&mut self, sources: &S, result: &mut I) {
        for frame in 0..sources.frames() {
            for source in 0..sources.sources() {
                let s = sources.get(frame, source);
                let position = Vector3::new(s.pos_x(), s.pos_y(), s.pos_z());
                let extent = Vector3::new(s.ext_x(), s.ext_y(), s.ext_z());
                let bal_unclamped = (position.x.simd_abs() - extent.x)
                    .simd_max(T::zero())
                    .simd_copysign(position.x);
                let bal = bal_unclamped.simd_min(T::one());
                result.set(frame, source, 0, bal);
            }
        }
    }
}

/**
Interprets audio sources based on their direction from the center of the scene.

Directional stereo is more physically accurate in headphones
as opposed to typical stereo speaker setups.
*/
#[derive(Default, Clone, Copy)]
pub struct Directional<T> {
    phantom_data: PhantomData<T>,
}

impl<T: Default> InterpretationInfo for Directional<T> {
    type Interpretation = T;

    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<S, I, T> InterpretSources<S, I> for Directional<T>
where
    T: SimdRealField + Copy + Default,
    I: Interpretations<T>,
    S: Sources,
    S::Source: Position<T> + Extent<T>,
{
    /// NOTE: Extent has only been clumsily and incorrectly implemented.
    fn interpret_sources(&mut self, sources: &S, result: &mut I) {
        for frame in 0..sources.frames() {
            for source in 0..sources.sources() {
                let s = sources.get(frame, source);
                let position = Vector3::new(s.pos_x(), s.pos_y(), s.pos_z());
                let extent = Vector3::new(s.ext_x(), s.ext_y(), s.ext_z());
                let divergence = directional_cuboid_solve(position, extent);
                let mut pos_norm = position;
                pos_norm.normalize_mut();
                let converged_balance = pos_norm.x * divergence.x;
                result.set(frame, source, 0, converged_balance);
            }
        }
    }
}
fn directional_cuboid_solve<T: SimdRealField + Copy>(
    position: Vector3<T>,
    extent: Vector3<T>,
) -> Vector3<T> {
    let position_abs = simd_abs(position);
    let mut plus = position_abs + extent;
    for i in plus.iter_mut() {
        i.simd_max(T::zero());
    }
    let plus = simd_abs(plus.normalize());
    let mut minus = position_abs - extent;
    for i in minus.iter_mut() {
        i.simd_max(T::zero());
    }
    let minus = simd_abs(minus.normalize());
    let mut divergence = Vector3::zeros();
    for ((plus, minus), d) in plus.iter().zip(minus.iter()).zip(divergence.iter_mut()) {
        *d = plus.simd_min(*minus);
    }
    divergence
}

fn simd_abs<T: SimdRealField>(value: Vector3<T>) -> Vector3<T> {
    let mut result = Vector3::zeros();
    for (a, b) in result.iter_mut().zip(value.iter()) {
        *a = b.simd_abs();
    }
    result
}

/// The sample processor for the stereo rendering mode.
#[derive(Default)]
pub struct Processor<T, Law> {
    pan_law: Law,
    scalar: PhantomData<T>,
}
impl<T, Law> Processor<T, Law>
where
    Law: PanLaw<T>,
{
    pub fn new(pan_law: Law) -> Self {
        Self {
            pan_law,
            scalar: PhantomData,
        }
    }

    pub fn set_pan_law(&mut self, pan_law: Law) {
        self.pan_law = pan_law;
    }
}

impl<T, Law> DelaySamples for Processor<T, Law> {
    fn delay_samples(&self, _sample_rate: u32) -> usize {
        0
    }
}
impl<T, Law> OutputChannels for Processor<T, Law> {
    fn output_channels(&self) -> usize {
        2
    }
}

impl<T: Default, Law> InterpretationInfo for Processor<T, Law> {
    type Interpretation = T;

    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<T, Law, I, In, Out> ProcessSamples<I, In, Out> for Processor<T, Law>
where
    Law: PanLaw<T>,
    In: Input<T>,
    Out: Output<T>,
    I: Interpretations<T>,
    T: Float + FlipSign + FromPrimitive + Default,
{
    fn process_samples(&mut self, result: &I, input: &In, output: &mut Out) {
        for frame in 0..input.frames() {
            for source in 0..input.sources() {
                for channel in 0..2u8 {
                    let gain = gain(channel, *result.get(frame, source, 0));
                    let attenuated = self.pan_law.attenuate(gain);
                    let current_sample = input.sample(frame, source);
                    output.set_channel(frame, usize::from(channel), current_sample * attenuated);
                }
            }
        }
    }
}
impl<T, Law> AcceptableFrames for Processor<T, Law> {
    fn acceptable_frames(&self, _sample_rate: u32, frames: usize) -> usize {
        frames
    }
}
impl<T, Law> MinimumFrames for Processor<T, Law> {
    fn minimum_frames(&self, _sample_rate: u32) -> usize {
        1
    }
}
