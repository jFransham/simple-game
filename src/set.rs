pub trait Intersects {
    fn intersects(&self, other: &Self) -> bool;
}

pub trait Set: Sized + Intersects {
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Option<Self>;
    fn contains(&self, subset: &Self) -> bool;

    fn contained_by(&self, superset: &Self) -> bool {
        superset.contains(self)
    }
}

impl<T: Set> Intersects for T {
    fn intersects(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }
}
