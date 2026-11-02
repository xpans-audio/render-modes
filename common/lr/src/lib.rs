/*!
Shared traits and functions between Stereo and Headphones
rendering modes in the xpans Ecosystem
*/
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod std;

use num::{Float, FromPrimitive, traits::FloatConst};

/// -3 dB center attenuation with a square root taper
#[derive(Debug, Default)]
pub struct SquareRoot;

impl<T> PanLaw<T> for SquareRoot
where
    T: Float,
{
    fn attenuate(&self, gain: T) -> T {
        square_root(gain)
    }
}
/// -3 dB center attenuation with a square root taper
pub fn square_root<T: Float>(gain: T) -> T {
    gain.sqrt()
}

/// -6 dB center attenuation with a linear taper
#[derive(Debug, Default)]
pub struct Linear;

impl<T> PanLaw<T> for Linear {
    fn attenuate(&self, gain: T) -> T {
        linear(gain)
    }
}
/// -6 dB center attenuation with a linear taper
pub fn linear<T>(gain: T) -> T {
    gain
}

/// -3 dB center attenuation with a sine taper
#[derive(Debug, Default)]
pub struct Sine;

impl<T> PanLaw<T> for Sine
where
    T: Float + FloatConst,
{
    fn attenuate(&self, gain: T) -> T {
        sine(gain)
    }
}

/// -3 dB center attenuation with a sine taper
pub fn sine<T>(gain: T) -> T
where
    T: Float + FloatConst,
{
    (gain * T::FRAC_PI_2()).sin()
}

/// Applies the pan law to the gain.
pub trait PanLaw<T> {
    fn attenuate(&self, gain: T) -> T;
}

impl<T> PanLaw<T> for fn(T) -> T {
    fn attenuate(&self, gain: T) -> T {
        self(gain)
    }
}

/// Calculates the gain for a stereo channel given the L/R balance.
///
/// `channel` should be either `0` or `1`, where `0` is the left channel and
/// `1` is the right channel.
pub fn gain<T>(channel: u8, lr: T) -> T
where
    T: FlipSign + Float + FromPrimitive,
{
    let should_flip = channel == 0;
    let lr = T::flip_sign(should_flip, lr);

    (lr + T::one()) / T::from_f64(2.).unwrap()
}

/// Conditionally negates the provided value.
pub trait FlipSign {
    fn flip_sign(should_flip: bool, v: Self) -> Self;
}

macro_rules! impl_flip_sign {
    ($float:ty, $bits:ty) => {
        impl FlipSign for $float {
            fn flip_sign(should_flip: bool, v: Self) -> Self {
                let flip = (<$bits>::from(should_flip)) << ((core::mem::size_of::<Self>() * 8) - 1);
                <$float>::from_bits(v.to_bits() ^ flip)
            }
        }
    };
}

impl_flip_sign!(f32, u32);
impl_flip_sign!(f64, u64);

#[cfg(test)]
#[test]
fn flip_sign_works() {
    let v = 1.;

    assert_eq!(f32::flip_sign(true, v), -v);
    assert_eq!(f32::flip_sign(false, v), v);
    assert_eq!(f32::flip_sign(true, -v), v);
    assert_eq!(f32::flip_sign(false, -v), -v);

    let v = 1.;
    assert_eq!(f64::flip_sign(true, v), -v);
    assert_eq!(f64::flip_sign(false, v), v);
    assert_eq!(f64::flip_sign(true, -v), v);
    assert_eq!(f64::flip_sign(false, -v), -v);
}

#[cfg(test)]
#[test]
fn gain_center() {
    test_gain_both_sides(0., [0.5, 0.5]);
}

#[cfg(test)]
#[test]
fn gain_sides() {
    test_gain_both_sides(-1., [1.0, 0.0]);
}

#[cfg(test)]
#[test]
fn gain_half_sides() {
    test_gain(-0.5, [0.75, 0.25]);
}

#[cfg(test)]
fn test_gain<T>(lr: T, desired_gains: [T; 2])
where
    T: core::fmt::Debug + FlipSign + Float + FromPrimitive,
{
    let mut gains = [T::zero(); 2];
    for channel in 0u8..2 {
        gains[channel as usize] = gain(channel, lr);
    }
    for i in 0..2 {
        assert_eq!(gains[i], desired_gains[i]);
    }
}

#[cfg(test)]
fn test_gain_both_sides<T>(lr: T, desired_gains: [T; 2])
where
    T: core::fmt::Debug + FlipSign + Float + FromPrimitive,
{
    let mut reversed = desired_gains.clone();
    reversed.reverse();

    test_gain(lr, desired_gains);
    test_gain(-lr, reversed);
}
