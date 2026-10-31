/*!
Implementation of xpans' Headphones rendering mode
*/
pub mod distance;
pub mod pan_law;

use crate::distance::{DistanceCurve, normalize_distance};
use crate::pan_law::PanLaw;
use core::marker::PhantomData;
use nalgebra::{SimdRealField, Vector3};
use num::{Float, FromPrimitive, cast::AsPrimitive};
use xpans::{Extent, Position};
use xpans_common_lr::{FlipSign, gain};
use xpans_render::mode::*;

/// The source interpreter for the headphone rendering mode.
#[derive(Default)]
pub struct Interpreter<T> {
    phantom_data: PhantomData<T>,
}
impl<T> Interpreter<T> {
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

/// The sample processor for the headphone rendering mode.
#[derive(Default)]
pub struct Processor<T, Law, D> {
    max_itd_nanos: u32,
    distance_curve: D,
    distance_effect: T,
    min_distance: T,
    max_distance: T,
    pan_law: Law,
}
impl<T: Float, Law, D: DistanceCurve<T>> Processor<T, Law, D> {
    /// Creates a new headphone sample processor.
    pub fn new(
        pan_law: Law,
        max_itd_nanos: u32,
        distance_curve: D,
        distance_effect: T,
        min_distance: T,
        max_distance: T,
    ) -> Self {
        Self {
            pan_law,
            max_itd_nanos,
            distance_curve,
            distance_effect,
            min_distance,
            max_distance,
        }
    }
    pub fn pan_law(&self) -> &Law {
        &self.pan_law
    }
    pub fn set_pan_law(&mut self, pan_law: Law) {
        self.pan_law = pan_law
    }
    pub fn max_itd_nanos(&self) -> u32 {
        self.max_itd_nanos
    }
    pub fn set_max_itd_nanos(&mut self, max_itd_nanos: u32) {
        self.max_itd_nanos = max_itd_nanos
    }
    pub fn distance_effect(&self) -> T {
        self.distance_effect
    }
    pub fn set_distance_effect(&mut self, distance_effect: T) {
        self.distance_effect = distance_effect
    }
    pub fn min_distance(&self) -> T {
        self.min_distance
    }
    pub fn set_min_distance(&mut self, min_distance: T) {
        self.min_distance = min_distance;
    }
    pub fn max_distance(&self) -> T {
        self.max_distance
    }
    pub fn set_max_distance(&mut self, max_distance: T) {
        self.max_distance = max_distance;
    }
    pub fn distance_curve(&self) -> &D {
        &self.distance_curve
    }
    pub fn set_distance_curve(&mut self, distance_curve: D) {
        self.distance_curve = distance_curve;
    }
    fn normalize_distance(&self, distance: T) -> T {
        normalize_distance(self.min_distance, self.max_distance, distance)
    }
    fn apply_distance_curve(&self, distance: T) -> T {
        self.distance_curve.distance_curve(distance)
    }
}

/// The interpretation type for the headphone rendering mode.
#[derive(Debug, Default, Clone, Copy)]
pub struct Interpretation<T> {
    lr: T,
    bf: T,
    du: T,
    distance: T,
}

impl<T> Interpretation<T> {
    pub fn lr(&self) -> &T {
        &self.lr
    }

    pub fn bf(&self) -> &T {
        &self.bf
    }

    pub fn du(&self) -> &T {
        &self.du
    }

    pub fn distance(&self) -> &T {
        &self.distance
    }
}

impl<T: Default> InterpretationInfo for Interpreter<T> {
    type Interpretation = Interpretation<T>;

    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<S, I, T> InterpretSources<S, I> for Interpreter<T>
where
    S: Sources,
    S::Source: Position<T> + Extent<T>,
    T: SimdRealField + Copy + Default + From<f32>,
    I: Interpretations<Interpretation<T>>,
{
    fn interpret_sources(&mut self, sources: &S, interpretations: &mut I) {
        for frame in 0..sources.frames() {
            for source in 0..sources.sources() {
                let src = sources.get(frame, source);
                let position = Vector3::new(src.pos_x(), src.pos_y(), src.pos_z());

                let pos_norm = position.normalize();

                let interp = interpretations.get_mut(frame, source, 0);

                interp.lr = pos_norm.x;
                interp.bf = pos_norm.y;
                interp.du = pos_norm.z;
                let distance = position.magnitude();
                interp.distance = distance;
            }
        }
    }
}

impl<T: Default, Law, D> InterpretationInfo for Processor<T, Law, D> {
    type Interpretation = Interpretation<T>;

    fn interpretation_length(&self) -> usize {
        1
    }
}
impl<T, Law, D> DelaySamples for Processor<T, Law, D> {
    fn delay_samples(&self, sample_rate: u32) -> usize {
        calculate_delay_samples(self.max_itd_nanos, sample_rate)
    }
}
impl<T, Law, D> OutputChannels for Processor<T, Law, D> {
    fn output_channels(&self) -> usize {
        2
    }
}

impl<T, Law, I, In, Out, D> ProcessSamples<I, In, Out> for Processor<T, Law, D>
where
    Law: PanLaw<T>,
    In: FractionalIndex<T, T> + SampleRate,
    Out: Output<T>,
    T: Float + FlipSign + FromPrimitive + Default + 'static,
    u32: AsPrimitive<T>,
    D: DistanceCurve<T>,
    I: Interpretations<Interpretation<T>>,
{
    fn process_samples(&mut self, interpretations: &I, input: &In, output: &mut Out) {
        for frame in 0..input.frames() {
            for source in 0..input.sources() {
                let interpretation = interpretations.get(frame, source, 0);
                let delay_samples = get_delay_samples(self.max_itd_nanos(), interpretation, input);
                let delayed_sample = input.fractional_sample(frame, source, delay_samples);
                let current_sample = input.sample(frame, source);
                let sample_pair = [current_sample, delayed_sample];
                for channel in 0u8..2 {
                    let is_delayed_channel = (channel == 0) == interpretation.lr.is_sign_positive();
                    let sample = sample_pair[is_delayed_channel as usize];
                    let distance = self.normalize_distance(interpretation.distance);
                    let dm = self.apply_distance_curve(distance) * self.distance_effect();
                    let divergence = T::one() - dm;
                    let gain: T = gain(channel, interpretation.lr * divergence);
                    let gain_attenuated = self.pan_law.attenuate(gain);
                    output.set_channel(frame, usize::from(channel), sample * gain_attenuated);
                }
            }
        }
    }
}

impl<T, Law, D> AcceptableFrames for Processor<T, Law, D> {
    fn acceptable_frames(&self, _sample_rate: u32, frames: usize) -> usize {
        frames
    }
}

impl<T, Law, D> MinimumFrames for Processor<T, Law, D> {
    fn minimum_frames(&self, _sample_rate: u32) -> usize {
        1
    }
}

fn get_delay_samples<T>(
    max_itd_nanos: u32,
    result: &Interpretation<T>,
    input: &impl SampleRate,
) -> T
where
    T: Float + 'static,
    u32: AsPrimitive<T>,
{
    let max_itd: T = max_itd_nanos.as_();
    let delay_ns = (result.lr * max_itd).abs();
    let sample_rate = input.sample_rate();
    nanos_to_samples(delay_ns, sample_rate)
}

const NANOS_PER_SEC: u32 = 1_000_000_000;

fn nanos_to_samples<T>(nanos: T, sample_rate: u32) -> T
where
    T: Float + 'static,
    u32: AsPrimitive<T>,
{
    (nanos * sample_rate.as_()) / (NANOS_PER_SEC.as_())
}

fn nanos_to_samples_int(nanos: u32, sample_rate: u32) -> usize {
    let nanos = u64::from(nanos);
    let sample_rate = u64::from(sample_rate);
    let samples = (nanos * sample_rate) / 1_000_000_000;
    (samples as usize).saturating_add(1)
}

/// Returns the number of buffered samples required for the given maximum ITD
/// and sample rate.
pub fn calculate_delay_samples(max_itd_nanos: u32, sample_rate: u32) -> usize {
    nanos_to_samples_int(max_itd_nanos, sample_rate)
}
