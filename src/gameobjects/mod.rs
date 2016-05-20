pub mod player;
pub mod main_menu;
pub mod background;

use ::set::Set;

use std::convert::TryFrom;
use std::ops::{Add, Sub};
use std::num::Zero;
use sdl2::rect::Rect as SdlRect;

pub type Bounds = Rectangle<f64>;
pub type Clip = Rectangle<u32>;
pub type Dest = Rectangle<i32, u32>;

#[derive(Clone, Copy, Debug, PartialEq)]
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

trait MinMax {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
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

impl Default for Bounds {
    fn default() -> Self {
        use std::f64;

        Bounds {
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

impl TryFrom<Bounds> for SdlRect {
    type Err = ();

    fn try_from(bounds: Bounds) -> Result<SdlRect, ()> {
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

impl TryFrom<Bounds> for Clip {
    type Err = ();

    fn try_from(bounds: Bounds) -> Result<Clip, ()> {
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

impl TryFrom<Bounds> for Dest {
    type Err = ();

    fn try_from(bounds: Bounds) -> Result<Dest, ()> {
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
