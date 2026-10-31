use std::marker::PhantomData;
use xpans_render::mode::*;

/// The source interpreter for the mono rendering mode.
#[derive(Debug, Default)]
pub struct Interpreter;

impl InterpretationInfo for Interpreter {
    type Interpretation = ();

    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<Source, I> InterpretSources<Source, I> for Interpreter
where
    I: Interpretations<()>,
{
    fn interpret_sources(&mut self, _source: &Source, _result: &mut I) {}
}

/// The sample processor for the mono rendering mode.
#[derive(Debug, Default, Clone, Copy)]
pub struct Processor<S>
where
    S: Copy,
{
    channels: usize,
    phantom_sample: PhantomData<S>,
}

impl<S: Copy> InterpretationInfo for Processor<S> {
    type Interpretation = ();

    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<S> Processor<S>
where
    S: Copy,
{
    pub fn new(channels: usize) -> Self {
        Self {
            channels,
            phantom_sample: PhantomData,
        }
    }
}

impl<S> DelaySamples for Processor<S>
where
    S: Copy,
{
    fn delay_samples(&self, _sample_rate: u32) -> usize {
        0
    }
}
impl<S> OutputChannels for Processor<S>
where
    S: Copy,
{
    fn output_channels(&self) -> usize {
        self.channels
    }
}

impl<S, I, In, Out> ProcessSamples<I, In, Out> for Processor<S>
where
    I: Interpretations<()>,
    In: Input<S>,
    Out: Output<S>,
    S: Copy,
{
    fn process_samples(&mut self, _result: &I, input: &In, output: &mut Out) {
        for frame in 0..input.frames() {
            for source in 0..input.sources() {
                let sample = input.sample(frame, source);
                for channel in 0..self.output_channels() {
                    output.set_channel(frame, channel, sample);
                }
            }
        }
    }
}
impl<S: Copy> AcceptableFrames for Processor<S> {
    fn acceptable_frames(&self, _sample_rate: u32, frames: usize) -> usize {
        frames
    }
}

impl<S: Copy> MinimumFrames for Processor<S> {
    fn minimum_frames(&self, _sample_rate: u32) -> usize {
        1
    }
}
