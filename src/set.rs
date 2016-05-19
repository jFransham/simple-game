pub trait Set: Sized {
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Option<Self>;
    fn contains(&self, subset: &Self) -> bool;

    fn contained_by(&self, superset: &Self) -> bool {
        superset.contains(self)
    }

    fn intersects(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }
}
