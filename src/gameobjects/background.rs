use ::time::*;
use ::set::*;
use ::gameobjects::*;
use ::graphics::sprites::{GetSize, Sprite};

use itertools::*;

pub struct ParallaxSet<
    Time: TimeExtensions,
    T: GetSize,
    I
> where for<'a> &'a I: IntoIterator<Item=&'a ([f64; 2], [f64; 2], Sprite<T>)> {
    pub start_time: Time,
    pub sprites: I,
}

impl<
    Time: Copy + TimeExtensions,
    T: GetSize,
    I
> ParallaxSet<Time, T, I> where
    for<'a> &'a I: IntoIterator<Item=&'a ([f64; 2], [f64; 2], Sprite<T>)>
{
    pub fn new(sprites: I, start: Time) -> Self {
        ParallaxSet {
            sprites: sprites,
            start_time: start,
        }
    }

    pub fn get_offsets(&self, now: Time) -> Vec<(&Sprite<T>, [i32; 2])> {
        let dt = now.exact_seconds() - self.start_time.exact_seconds();

        self.sprites.into_iter().map(
            |&([vx, vy], [x, y], ref s)| {
                let (dx, dy) = (dt * vx, dt * vy);

                (s, [(x + dx) as _, (y + dy) as _])
            }
        ).collect()
    }

    pub fn get_destinations(
        &self,
        screen: Dest,
        now: Time
    ) -> Vec<(Sprite<T>, Dest)> {
        let dt = now.exact_seconds() - self.start_time.exact_seconds();

        let (screen_w, screen_h) = (screen.width, screen.height);

        self.sprites.into_iter().map(
            |&([vx, vy], [x, y], ref s)| {
                let (dx, dy) = (dt * vx, dt * vy);

                (s, [(x + dx) as _, (y + dy) as _])
            }
        ).flat_map(|(spr, [offset_x, offset_y])| {
            let (spr_w, spr_h) = (spr.mask.width, spr.mask.height);

            (offset_x..screen_w as i32).step_by(spr_w as i32)
                .cartesian_product(
                    (offset_y..screen_h as i32).step_by(spr_h as i32)
                )
                .map(move |(x, y)|
                     (
                         spr.clone(),
                         Dest {
                             x: x,
                             y: y,
                             width: spr_w,
                             height: spr_h,
                         },
                     )
                )
                .filter(|&(_, ref c)| screen.intersects(c))
        }).collect()
    }
}
