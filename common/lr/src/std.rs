use crate::PanLaw;

impl<V, T> PanLaw<V> for Box<T>
where
    T: PanLaw<V> + ?Sized,
{
    fn attenuate(&self, gain: V) -> V {
        self.as_ref().attenuate(gain)
    }
}
