use std::ops::AddAssign;

use crate::{
    distance::DistanceSettings,
    sofa::Sofa,
    upols::{FftNum, Upols, Value},
};
use num::{Complex, Float, cast::AsPrimitive};
use xpans_common_lr::{PanLaw, gain};
use xpans_render::mode::{
    AcceptableFrames, DelaySamples, FractionalIndex, InterpretationInfo, Interpretations,
    MinimumFrames, Output, OutputChannels, ProcessSamples, SampleRate,
};

use crate::{
    Interpretation,
    distance::{DistanceCurve, normalize_distance},
};

/// Uses Uniformly Partitioned Overlap Save convolution
/// from filters from a SOFA file
pub struct Processor<T, Law, D>
where
    T: FftNum,
{
    sofa: Sofa,
    upols: Upols<T>,
    distance_curve: D,
    distance_effect: T,
    min_distance: T,
    max_distance: T,
    pan_law: Law,
    block_size: usize,
}
impl<T, Law, D> Processor<T, Law, D>
where
    T: Float + Value,
    Complex<T>: AddAssign,
    D: DistanceCurve<T>,
    usize: AsPrimitive<T>,
{
    /// Creates a new headphone sample processor.
    pub fn new(
        sofa: Sofa,
        source_count: usize,
        block_size: usize,
        pan_law: Law,
        distance_curve: D,
        distance_settings: DistanceSettings<T>,
    ) -> Self {
        let filter_length = sofa.filter_length();
        let upols = Upols::new(source_count, filter_length, block_size);
        Self {
            sofa,
            upols,
            pan_law,
            distance_curve,
            distance_effect: *distance_settings.distance_effect(),
            min_distance: *distance_settings.min_distance(),
            max_distance: *distance_settings.max_distance(),
            block_size,
        }
    }
    pub fn pan_law(&self) -> &Law {
        &self.pan_law
    }
    pub fn set_pan_law(&mut self, pan_law: Law) {
        self.pan_law = pan_law
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

impl<T: FftNum, Law, D> DelaySamples for Processor<T, Law, D> {
    fn delay_samples(&self, _sample_rate: u32) -> usize {
        0
    }
}
impl<T: FftNum, Law, D> OutputChannels for Processor<T, Law, D> {
    fn output_channels(&self) -> usize {
        2
    }
}

impl<I, Law, In, Out, D> ProcessSamples<I, In, Out> for Processor<f32, Law, D>
where
    f32: Value,
    I: Interpretations<Interpretation<f32>>,
    Law: PanLaw<f32>,
    In: FractionalIndex<f32, f32> + SampleRate,
    Out: Output<f32>,
    // T: Float + FlipSign + FromPrimitive + Default + 'static + FftNum + ConstZero,
    u32: AsPrimitive<f32>,
    D: DistanceCurve<f32>,
{
    fn process_samples(&mut self, result: &I, input: &In, output: &mut Out) {
        for source in 0..input.sources() {
            let input_block = self.upols.audio_input_mut(source).iter_mut();
            for (frame, sample) in input_block.enumerate() {
                *sample = input.sample(frame, source)
            }
            self.upols.push_audio_input(source);

            let interpretation = result.get(0, source, 0);

            let [filter_l, filter_r] =
                self.sofa
                    .filters([interpretation.bf, -interpretation.lr, interpretation.du]);

            self.upols.update_filter(source, 0, filter_l);
            self.upols.update_filter(source, 1, filter_r);
        }

        for source in 0..input.sources() {
            for channel in 0u8..2 {
                self.upols.convolve(source, channel as usize);
                let filtered_output = self.upols.output().iter();
                for (frame, sample) in filtered_output.enumerate() {
                    let interpretation = result.get(frame, source, 0);
                    let distance = self.normalize_distance(interpretation.distance);
                    let dm = self.apply_distance_curve(distance) * self.distance_effect();
                    let divergence = 1.0 - dm;
                    let gain: f32 = gain(channel, interpretation.lr * divergence);
                    let gain_attenuated = self.pan_law.attenuate(gain);

                    let sample = sample * gain_attenuated;

                    output.set_channel(frame, channel as usize, sample);
                }
            }
        }
        self.upols.advance_audio_input();
    }
}

impl<T: FftNum, Law, D> AcceptableFrames for Processor<T, Law, D> {
    fn acceptable_frames(&self, _sample_rate: u32, frames: usize) -> usize {
        // TODO:
        // frames - remainder_pow2(frames, self.block_size)
        let able = frames >= self.block_size;
        usize::from(able) * self.block_size
    }
}
#[allow(unused)]
#[inline]
fn remainder_pow2(lhs: usize, rhs: usize) -> usize {
    (rhs - 1) & lhs
}
impl<T: FftNum, Law, D> MinimumFrames for Processor<T, Law, D> {
    fn minimum_frames(&self, _sample_rate: u32) -> usize {
        self.block_size
    }
}

impl<T: Default + FftNum, Law, D> InterpretationInfo for Processor<T, Law, D> {
    type Interpretation = Interpretation<T>;

    fn interpretation_length(&self) -> usize {
        1
    }
}
