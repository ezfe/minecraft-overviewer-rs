/// Trait for coordinate types that support painters algorithm iteration
/// (back-to-front, bottom-to-top for proper isometric rendering)
pub trait PaintersRange: Sized {
    type Iter: Iterator<Item = Self>;

    fn painters_range_to(&self, other: &Self) -> Self::Iter;
}
