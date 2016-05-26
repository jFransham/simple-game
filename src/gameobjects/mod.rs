pub mod player;
pub mod main_menu;
pub mod background;

use ::set::{Set, Intersects};

use std::convert::{TryFrom, TryInto};
use std::ops::{Add, Sub};
use std::num::Zero;
use sdl2::rect::Rect as SdlRect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bounds {
    Rectangle(BoundingRect),
    Circle(Circle<f64>),
}

impl Bounds {
    pub fn left(&self) -> f64 {
        use self::Bounds::*;

        match *self {
            Rectangle(ref b) => b.left(),
            Circle(ref c) => c.left(),
        }
    }

    pub fn right(&self) -> f64 {
        use self::Bounds::*;

        match *self {
            Rectangle(ref b) => b.right(),
            Circle(ref c) => c.right(),
        }
    }

    pub fn top(&self) -> f64 {
        use self::Bounds::*;

        match *self {
            Rectangle(ref b) => b.top(),
            Circle(ref c) => c.top(),
        }
    }

    pub fn bottom(&self) -> f64 {
        use self::Bounds::*;

        match *self {
            Rectangle(ref b) => b.bottom(),
            Circle(ref c) => c.bottom(),
        }
    }
}

fn rectangle_intersects_circle(c: &Circle<f64>, r: &Rectangle<f64>) -> bool {
    let (test_x, test_y) = (
        c.x.limit(r.left(), r.right()),
        c.y.limit(r.top(), r.bottom()),
    );

    let (dx, dy) = (c.x - test_x, c.y - test_y);

    let dist_squared = dx * dx + dy * dy;

    dist_squared < c.radius * c.radius
}

impl Intersects for Bounds {
    fn intersects(&self, other: &Self) -> bool {
        use self::Bounds::*;

        match (*self, *other) {
            (Rectangle(ref a), Rectangle(ref b)) => a.intersects(b),
            (Circle(ref a), Circle(ref b)) => a.intersects(b),
            (
                Rectangle(ref a), Circle(ref b)
            ) | (
                Circle(ref b), Rectangle(ref a)
            ) => rectangle_intersects_circle(b, a),
        }
    }
}

impl Intersects for Circle<f64> {
    fn intersects(&self, other: &Self) -> bool {
        let (dx, dy) = (self.x - other.x, self.y - other.y);
        let dist_squared = dx * dx + dy * dy;
        let total_rad = self.radius + other.radius;
        let total_rad_squared = total_rad * total_rad;

        dist_squared < total_rad_squared
    }
}

impl From<Rectangle<f64>> for Bounds {
    fn from(r: Rectangle<f64>) -> Self {
        Bounds::Rectangle(r)
    }
}

impl From<Circle<f64>> for Bounds {
    fn from(c: Circle<f64>) -> Self {
        Bounds::Circle(c)
    }
}

pub type BoundingRect = Rectangle<f64>;
pub type Clip = Rectangle<u32>;
pub type Dest = Rectangle<i32, u32>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Circle<P, S=P> {
    pub x: P,
    pub y: P,
    pub radius: S,
}

impl<
    T: Copy + Add<T, Output=T> + Sub<T, Output=T>
> Circle<T> {
    pub fn left(&self) -> T { self.x - self.radius }
    pub fn right(&self) -> T { self.x + self.radius }
    pub fn top(&self) -> T { self.y - self.radius }
    pub fn bottom(&self) -> T { self.y + self.radius }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle<P, S=P> {
    pub x: P,
    pub y: P,
    pub width: S,
    pub height: S,
}

impl<P: Clone, S: Clone> Rectangle<P, S> {
    pub fn with_position(&self, x: P, y: P) -> Self {
        Rectangle {
            x: x,
            y: y,
            .. self.clone()
        }
    }

    pub fn with_size(&self, w: S, h: S) -> Self {
        Rectangle {
            width: w,
            height: h,
            .. self.clone()
        }
    }
}

#[allow(overlapping_inherent_impls)]
impl<O, P: Copy + Add<S, Output=O>, S: Copy> Rectangle<P, S> {
    pub fn left(&self) -> P { self.x }
    pub fn right(&self) -> O { self.x + self.width }
    pub fn top(&self) -> P { self.y }
    pub fn bottom(&self) -> O { self.y + self.height }
}

#[allow(overlapping_inherent_impls)]
impl Dest {
    pub fn left(&self) -> i32 { self.x }
    pub fn right(&self) -> i32 { self.x + self.width as i32 }
    pub fn top(&self) -> i32 { self.y }
    pub fn bottom(&self) -> i32 { self.y + self.height as i32 }
}

pub trait MinMax: Sized {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn limit(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }
}

impl<T: PartialOrd> MinMax for T {
    fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl<O, P: Copy + Add<P, Output=O>, S: Copy> Rectangle<P, S> {
    pub fn with_offset(&self, x: P, y: P) -> Rectangle<O, S> {
        Rectangle {
            x: self.x + x,
            y: self.y + y,
            width: self.width,
            height: self.height,
        }
    }
}

impl<
    T: Add<T, Output=T> + Sub<T, Output=T> + PartialOrd + Copy
> Rectangle<T> {
    pub fn move_inside(&self, boundary: &Self) -> Option<Self> {
        if self.width > boundary.width || self.height > boundary.height {
            None
        } else {
            Some(
                Rectangle {
                    x: self.x
                        .max(boundary.x)
                        .min(boundary.right() - self.width),
                    y: self.y
                        .max(boundary.y)
                        .min(boundary.bottom() - self.height),
                    width: self.width,
                    height: self.height,
                }
            )
        }
    }
}

impl Default for BoundingRect {
    fn default() -> Self {
        use std::f64;

        BoundingRect {
            x: 0.0,
            y: 0.0,
            width: f64::NAN,
            height: f64::NAN,
        }
    }
}

impl<P: Default, S: Default> Default for Rectangle<P, S> {
    default fn default() -> Self {
        Rectangle {
            x: Default::default(),
            y: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl Set for Dest {
    fn union(&self, other: &Self) -> Self {
        let (x, y) = (
            self.x.min(other.x),
            self.y.min(other.y),
        );

        Dest {
            x: x,
            y: y,
            width: (self.right().max(other.right()) - x) as _,
            height: (self.bottom().max(other.bottom()) - y) as _,
        }
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        let (x, y) = (
            self.x.max(other.x),
            self.y.max(other.y),
        );

        let (w, h) = (
            self.right().min(other.right()) - x,
            self.bottom().min(other.bottom()) - y,
        );

        if w > 0 && h > 0 {
            Some(
                Dest {
                    x: x,
                    y: y,
                    width: w as _,
                    height: h as _,
                }
            )
        } else {
            None
        }
    }

    fn contains(&self, subset: &Self) -> bool {
        subset.left() >= self.left() &&
        subset.right() <= self.right() &&
        subset.top() >= self.top() &&
        subset.bottom() <= self.bottom()
    }
}

impl<
    T: Add<T, Output=T> + Sub<T, Output=T> + Zero + PartialOrd + Copy
> Set for Rectangle<T> {
    fn union(&self, other: &Self) -> Self {
        let (x, y) = (
            self.x.min(other.x),
            self.y.min(other.y),
        );

        Rectangle {
            x: x,
            y: y,
            width: self.right().max(other.right()) - x,
            height: self.bottom().max(other.bottom()) - y,
        }
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        let (x, y) = (
            self.x.max(other.x),
            self.y.max(other.y),
        );

        let (w, h) = (
            self.right().min(other.right()) - x,
            self.bottom().min(other.bottom()) - y,
        );

        if w > T::zero() && h > T::zero() {
            Some(
                Rectangle {
                    x: x,
                    y: y,
                    width: w,
                    height: h,
                }
            )
        } else {
            None
        }
    }

    fn contains(&self, subset: &Self) -> bool {
        subset.left() >= self.left() &&
        subset.right() <= self.right() &&
        subset.top() >= self.top() &&
        subset.bottom() <= self.bottom()
    }
}

impl TryFrom<BoundingRect> for SdlRect {
    type Err = ();

    fn try_from(bounds: BoundingRect) -> Result<SdlRect, ()> {
        use std::i32::MAX as IMAX;
        use std::u32::MAX as UMAX;
        let (imax, umax) = (IMAX as f64, UMAX as f64);

        if
            bounds.x.abs() <= imax &&
            bounds.y.abs() <= imax &&
            bounds.width <= umax &&
            bounds.height <= umax &&
            bounds.width > 0.0 &&
            bounds.height > 0.0
        {
            Ok(
                SdlRect::new(
                    bounds.x as _,
                    bounds.y as _,
                    bounds.width as _,
                    bounds.height as _
                )
            )
        } else {
            Err(())
        }
    }
}

impl TryFrom<Bounds> for SdlRect {
    type Err = ();

    fn try_from(bounds: Bounds) -> Result<Self, ()> {
        match bounds {
            Bounds::Rectangle(rect) => rect.try_into(),
            _ => Err(()),
        }
    }
}

impl TryFrom<Bounds> for Clip {
    type Err = ();

    fn try_from(bounds: Bounds) -> Result<Self, ()> {
        match bounds {
            Bounds::Rectangle(rect) => rect.try_into(),
            _ => Err(()),
        }
    }
}

impl TryFrom<Bounds> for Dest {
    type Err = ();

    fn try_from(bounds: Bounds) -> Result<Self, ()> {
        match bounds {
            Bounds::Rectangle(rect) => rect.try_into(),
            _ => Err(()),
        }
    }
}

impl TryFrom<BoundingRect> for Clip {
    type Err = ();

    fn try_from(bounds: BoundingRect) -> Result<Clip, ()> {
        use std::i32::MAX as IMAX;
        use std::u32::MAX as UMAX;
        let (imax, umax) = (IMAX as f64, UMAX as f64);

        if
            bounds.x.abs() <= imax &&
            bounds.y.abs() <= imax &&
            bounds.width <= umax &&
            bounds.height <= umax &&
            bounds.width > 0.0 &&
            bounds.height > 0.0
        {
            Ok(
                Clip {
                    x: bounds.x as _,
                    y: bounds.y as _,
                    width: bounds.width as _,
                    height: bounds.height as _
                }
            )
        } else {
            Err(())
        }
    }
}

impl TryFrom<BoundingRect> for Dest {
    type Err = ();

    fn try_from(bounds: BoundingRect) -> Result<Dest, ()> {
        use std::i32::MAX as IMAX;
        use std::u32::MAX as UMAX;
        let (imax, umax) = (IMAX as f64, UMAX as f64);

        if
            bounds.x.abs() <= imax &&
            bounds.y.abs() <= imax &&
            bounds.width <= umax &&
            bounds.height <= umax &&
            bounds.width > 0.0 &&
            bounds.height > 0.0
        {
            Ok(
                Dest {
                    x: bounds.x as _,
                    y: bounds.y as _,
                    width: bounds.width as _,
                    height: bounds.height as _
                }
            )
        } else {
            Err(())
        }
    }
}

impl From<Clip> for Dest {
    fn from(clip: Clip) -> Self {
        Rectangle {
            x: clip.x as i32,
            y: clip.y as i32,
            width: clip.width,
            height: clip.height,
        }
    }
}

impl From<Dest> for SdlRect {
    fn from(clip: Dest) -> SdlRect {
        SdlRect::new(
            clip.x,
            clip.y,
            clip.width,
            clip.height
        )
    }
}

impl From<Clip> for SdlRect {
    fn from(clip: Clip) -> SdlRect {
        SdlRect::new(
            clip.x as _,
            clip.y as _,
            clip.width,
            clip.height
        )
    }
}
