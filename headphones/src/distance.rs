//! Various distance curves
use num::{Float, traits::FloatConst};

/// Contains distance-related setting values
pub struct DistanceSettings<T> {
    distance_effect: T,
    min_distance: T,
    max_distance: T,
}

impl<T> DistanceSettings<T> {
    pub fn new(distance_effect: T, min_distance: T, max_distance: T) -> Self {
        Self {
            distance_effect,
            min_distance,
            max_distance,
        }
    }

    pub fn distance_effect(&self) -> &T {
        &self.distance_effect
    }

    pub fn min_distance(&self) -> &T {
        &self.min_distance
    }

    pub fn max_distance(&self) -> &T {
        &self.max_distance
    }

    pub fn distance_effect_mut(&mut self) -> &mut T {
        &mut self.distance_effect
    }

    pub fn min_distance_mut(&mut self) -> &mut T {
        &mut self.min_distance
    }

    pub fn max_distance_mut(&mut self) -> &mut T {
        &mut self.max_distance
    }

    pub fn set_distance_effect(&mut self, distance_effect: T) {
        self.distance_effect = distance_effect;
    }

    pub fn set_min_distance(&mut self, min_distance: T) {
        self.min_distance = min_distance;
    }

    pub fn set_max_distance(&mut self, max_distance: T) {
        self.max_distance = max_distance;
    }
}

#[derive(Debug, Default)]
pub struct Linear;
#[derive(Debug, Default)]
pub struct Exponential;
#[derive(Debug, Default)]
pub struct SquareRoot;
#[derive(Debug, Default)]
pub struct Sine;

/// Applies a curve to the *normalized* distance value.
///
/// The normalized distance is `1.0` if it is at or farther than the maximum
/// distance, and `0.0` if it is at or closer than the minimum distance.
pub trait DistanceCurve<T> {
    fn distance_curve(&self, distance: T) -> T;
}

impl<T> DistanceCurve<T> for Linear {
    fn distance_curve(&self, distance: T) -> T {
        linear(distance)
    }
}
pub fn linear<T>(distance: T) -> T {
    distance
}

impl<T: Float> DistanceCurve<T> for SquareRoot {
    fn distance_curve(&self, distance: T) -> T {
        square_root(distance)
    }
}
pub fn square_root<T: Float>(distance: T) -> T {
    distance.sqrt()
}

impl<T: Float + FloatConst> DistanceCurve<T> for Sine {
    fn distance_curve(&self, distance: T) -> T {
        sine(distance)
    }
}
pub fn sine<T: Float + FloatConst>(distance: T) -> T {
    (distance * T::FRAC_PI_2()).sin()
}

impl<T: Float> DistanceCurve<T> for Exponential {
    fn distance_curve(&self, distance: T) -> T {
        exponential(distance)
    }
}
pub fn exponential<T: Float>(distance: T) -> T {
    distance * distance
}

impl<T: Float> DistanceCurve<T> for fn(T) -> T {
    fn distance_curve(&self, distance: T) -> T {
        self(distance)
    }
}
impl<V, T: DistanceCurve<V> + ?Sized> DistanceCurve<V> for Box<T> {
    fn distance_curve(&self, distance: V) -> V {
        self.as_ref().distance_curve(distance)
    }
}

pub(crate) fn normalize_distance<T: Float>(min_distance: T, max_distance: T, distance: T) -> T {
    if min_distance > max_distance {
        return T::zero();
    }
    let v = max_distance - min_distance;
    let min = distance - min_distance;

    (min / v).clamp(T::zero(), T::one())
}

#[cfg(test)]
#[test]
fn normalize_half() {
    let normalized = normalize_distance(0.1, 1.1, 0.6);
    assert_eq!(normalized, 0.5);
}

#[cfg(test)]
#[test]
fn normalize_far() {
    let normalized = normalize_distance(0.1, 1.1, 2.0);
    assert_eq!(normalized, 1.0);
}

#[cfg(test)]
#[test]
fn normalize_near() {
    let normalized = normalize_distance(0.1, 1.1, 0.05);
    assert_eq!(normalized, 0.0);
}
