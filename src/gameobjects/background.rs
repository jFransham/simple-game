use ::time::*;
use ::graphics::sprites::{GetSize, Sprite};

pub struct ParallaxSet<
    Time: TimeExtensions,
    T: GetSize,
    I
> where for<'a> &'a I: IntoIterator<Item=&'a ([f64; 2], Sprite<T>)> {
    pub start_time: Time,
    pub sprites: I,
}

impl<
    Time: Copy + TimeExtensions,
    T: GetSize,
    I
> ParallaxSet<Time, T, I> where
    for<'a> &'a I: IntoIterator<Item=&'a ([f64; 2], Sprite<T>)>
{
    pub fn new(sprites: I, start: Time) -> Self {
        ParallaxSet {
            sprites: sprites,
            start_time: start,
        }
    }

    pub fn offset_sprites(&self, now: Time) -> Vec<(&Sprite<T>, [i32; 2])> {
        let dt = now.exact_seconds() - self.start_time.exact_seconds();

        self.sprites.into_iter().map(
            |&([vx, vy], ref s)| {
                let (dx, dy) = (dt * vx, dt * vy);

                (s, [dx as _, dy as _])
            }
        ).collect()
    }
}
