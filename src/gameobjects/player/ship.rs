use ::gameobjects::*;
use ::events::*;
use ::view::*;
use ::graphics::sprites::{
    VisibleComponent,
    VisibleRect,
    Sprite,
};
use ::time::*;
use ::set::Intersects;

use super::*;
use super::command_builder::CommandBuilder;

use std::convert::TryInto;
use std::collections::HashMap;
use sdl2::pixels::Color;
use sdl2::render::Texture;

/// The different states our ship might be in. In the image, they're ordered
/// from left to right, then from top to bottom.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ShipFrame {
    UpNorm,
    UpFast,
    UpSlow,
    MidNorm,
    MidFast,
    MidSlow,
    DownNorm,
    DownFast,
    DownSlow,
}

pub static ALL_FRAMES: [ShipFrame; 9] = [
    ShipFrame::UpNorm,
    ShipFrame::UpFast,
    ShipFrame::UpSlow,
    ShipFrame::MidNorm,
    ShipFrame::MidFast,
    ShipFrame::MidSlow,
    ShipFrame::DownNorm,
    ShipFrame::DownFast,
    ShipFrame::DownSlow,
];

pub struct GunArgs {
    pub bounds: Bounds,
}

pub trait Gun: Sized {
    fn spawn_bullets(
        &mut self,
        args: GunArgs,
        _: GameTime
    ) -> Vec<SimpleObject>;
    fn next_weapon(&mut self) {}
}

pub struct ShipGun {
    pub kind: BulletKind,
    pub sine: SineGun,
    pub standard: StandardGun,
}

impl Gun for ShipGun {
    fn spawn_bullets(
        &mut self,
        args: GunArgs,
        time: GameTime
    ) -> Vec<SimpleObject> {
        if self.kind == BulletKind::Sine {
            self.sine.spawn_bullets(args, time)
        } else {
            self.standard.spawn_bullets(args, time)
        }
    }

    fn next_weapon(&mut self) {
        self.kind = if self.kind == BulletKind::Sine {
            BulletKind::Standard
        } else {
            BulletKind::Sine
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Neg,
    Zero,
    Pos,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Zero
    }
}

enum ShipCommand {
    Move([Direction; 2]),
    Fire,
    NextWeapon,
}

pub struct Ship<G: Gun/*, C: CommandBuilder<Self, ShipCommand>*/> {
    pub bounds: BoundingRect,
    //pub command_builder: C,
    pub gun: G,
    pub dir: [Direction; 2],
    pub sprites: HashMap<ShipFrame, Sprite<Texture>>,
}

impl<G: Gun> Ship<G> {
    fn get_control(up: bool, down: bool, left: bool, right: bool) -> [Direction; 2] {
        use ::fixed_size_iter::FixedSizeIntoMap;
        use self::Direction::*;

        [(left, right), (up, down)].map(
            |(a, b)| match (a, b) {
                (true, true) | (false, false) => Zero,
                (true, _) => Neg,
                (_, true) => Pos,
            }
        )
    }

    fn movement_direction(dirs: [Direction; 2]) -> [f64; 2] {
        use self::Direction::*;
        use ::fixed_size_iter::FixedSizeIntoMap;

        Self::normalize(
            dirs.map(|d| match d { Neg => -1.0, Zero => 0.0, Pos => 1.0 })
        )
    }

    fn normalize([x, y]: [f64; 2]) -> [f64; 2] {
        match [x, y] {
            [0.0, 0.0] => [0.0, 0.0],
            [x, y] => {
                let len = (x*x + y*y).sqrt();

                [x / len, y / len]
            },
        }
    }

    #[allow(collapsible_if)]
    fn get_frame(vel: [Direction; 2]) -> ShipFrame {
        use self::ShipFrame::*;
        use self::Direction::*;
        use ::fixed_size_iter::FixedSizeIntoMap;

        match vel {
            [Neg,  Neg ] => UpSlow,
            [Neg,  Pos ] => DownSlow,
            [Neg,  Zero] => MidSlow,
            [Pos,  Neg ] => UpFast,
            [Pos,  Pos ] => DownFast,
            [Pos,  Zero] => MidFast,
            [Zero, Neg ] => UpNorm,
            [Zero, Pos ] => DownNorm,
            [Zero, Zero] => MidNorm,
        }
    }
}

impl<G: Gun> GameObject<Keys, Texture> for Ship<G> {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        time: GameTime
    ) -> Vec<GameAction<Keys, Texture>> {
        use ::fixed_size_iter::FixedSizeIntoMap;

        let player_speed = 230.0;

        let dt = time.elapsed.exact_seconds();

        let (sw, sh) = context.renderer.output_size().map(
            |(a, b)| (a as f64, b as f64)
        ).unwrap();

        self.dir = {
            let keys = &context.events.down;

            Self::get_control(
                keys.up,
                keys.down,
                keys.left,
                keys.right,
            )
        };

        let [dx, dy] = Self::movement_direction(self.dir).map(
            |a| a * dt * player_speed
        );

        self.bounds.x += dx;
        self.bounds.y += dy;

        self.bounds = self.bounds.move_inside(
            &BoundingRect {
                width: sw,
                height: sh,
                .. Default::default()
            }
        ).unwrap();

        if context.events.pressed.next_weapon {
            self.gun.next_weapon();
        }

        if context.events.down.fire {
            vec![
                GameAction::AddObjects(
                    self.gun.spawn_bullets(
                        GunArgs { bounds: self.bounds.into(), },
                        time
                    )
                )
            ]
        } else {
            vec![]
        }
    }

    fn sprites(&self, _: GameTime)
        -> Vec<(VisibleComponent<Texture>, Dest)>
    {
        vec![
            (
                self.sprites[&Self::get_frame(self.dir)].clone().into(),
                self.bounds.try_into().unwrap(),
            )
        ]
    }

    fn bounds(&self) -> Option<Bounds> { Some(self.bounds.into()) }
}

pub struct SineGun {
    pub last_ammo_at: u32,
    pub last_shot_at: u32,
    pub ammo_intervals: [u32; 3],
    pub ammo: u8,
    pub max_ammo: u8,
}

impl SineGun {
    pub fn new(now: u32) -> SineGun {
        let max = 10;

        SineGun {
            last_ammo_at: now,
            last_shot_at: 0,
            ammo_intervals: [1000, 700, 400],
            ammo: max,
            max_ammo: max,
        }
    }

    fn get_interval(&self) -> u32 {
        use ::gameobjects::MinMax;

        self.ammo_intervals[
            (self.ammo_intervals.len() - 1).min(self.ammo as _)
        ]
    }
}

impl Gun for SineGun {
    fn spawn_bullets(
        &mut self,
        args: GunArgs,
        time: GameTime
    ) -> Vec<SimpleObject> {
        use ::gameobjects::MinMax;

        {
            let mut time_diff = time.total - self.last_ammo_at;
            let mut interval = self.get_interval();

            while time_diff >= interval {
                self.last_ammo_at = time.total;

                time_diff -= interval;

                self.ammo = (self.ammo as u32 + 1).min(self.max_ammo as _) as _;

                interval = self.get_interval();
            }
        }

        {
            let time_diff = time.total - self.last_shot_at;

            if time_diff >= 80 {
                self.last_shot_at = time.total;
            } else {
                return vec![];
            }
        }

        if self.ammo == 0 {
            return vec![];
        } else {
            self.ammo -= 1;
        }

        self.last_ammo_at = time.total;

        let cannons_x = args.bounds.left() + 30.0;
        let cannon1_y = args.bounds.top() + 6.0;
        let cannon2_y = args.bounds.bottom() - 10.0;

        vec![
            box SineBullet::new(
                [cannons_x, cannon1_y],
                -90.0,
                time.total
            ),
            box SineBullet::new(
                [cannons_x, cannon2_y],
                90.0,
                time.total
            ),
        ]
    }
}

pub struct StandardGun {
    pub last_shot_at: u32,
}

impl StandardGun {
    pub fn new(now: u32) -> StandardGun {
        StandardGun {
            last_shot_at: now,
        }
    }
}

impl Gun for StandardGun {
    fn spawn_bullets(
        &mut self,
        args: GunArgs,
        time: GameTime
    ) -> Vec<SimpleObject> {
        let time_diff = time.total - self.last_shot_at;

        if time_diff < 400 {
            return vec![];
        }

        self.last_shot_at = time.total;

        let cannons_x = args.bounds.left() + 30.0;
        let cannon1_y = args.bounds.top() + 6.0;
        let cannon2_y = args.bounds.bottom() - 10.0;

        vec![
            box Bullet::new(
                [cannons_x, cannon1_y],
            ),
            box Bullet::new(
                [cannons_x, cannon2_y],
            ),
        ]
    }
}

pub struct SineBullet {
    pub bounds: BoundingRect,
    pub born_at: u32,
    pub angular_velocity: f64,
    pub origin_y: f64,
    pub amplitude: f64,
}

impl SineBullet {
    pub fn new([x, y]: [f64; 2], amplitude: f64, now: u32) -> SineBullet {
        SineBullet {
            amplitude: amplitude,
            born_at: now,
            angular_velocity: 4.0,
            bounds: BoundingRect {
                x: x,
                y: y,
                width: 8.0,
                height: 4.0,
            },
            origin_y: y,
        }
    }
}

impl GameObject<Keys, Texture> for SineBullet {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        time: GameTime
    ) -> Vec<GameAction<Keys, Texture>> {
        let velocity_x = 270.0;

        let time_alive = time.total - self.born_at;

        let (elapsed, alive_secs) = (
            time.elapsed.exact_seconds(),
            time_alive.exact_seconds(),
        );

        self.bounds.x += velocity_x * elapsed;
        self.bounds.y = self.origin_y + self.amplitude * (
            self.angular_velocity * alive_secs
        ).sin();

        let screen = context.renderer.output_size().map(
            |(w, h)| BoundingRect::default().with_size(w as _, h as _)
        ).unwrap();

        if
            self.bounds.left() > screen.left() &&
            self.bounds.right() < screen.right()
        {
            vec![]
        } else {
            vec![GameAction::Delete]
        }
    }

    fn sprites(&self, _: GameTime)
        -> Vec<(VisibleComponent<Texture>, Dest)>
    {
        vec![
            (
                VisibleRect(Color::RGB(230, 30, 30)).into(),
                self.bounds.try_into().unwrap(),
            )
        ]
    }

    fn bounds(&self) -> Option<Bounds> { Some(self.bounds.into()) }

    fn on_hit(&self) -> Option<GameMessage<Keys, Texture>> {
        Some(
            GameMessage::Hit {
                other: self as _,
                info: DamageInfo {
                    damage: 20,
                    filter: DamageFilter::Player,
                },
            }
        )
    }

    fn receive_message<'a>(
        &'a mut self,
        ctx: &mut Context<Keys>,
        time: GameTime,
        m: GameMessage<'a, Keys, Texture>
    ) -> Vec<GameAction<Keys, Texture>> {
        if let GameMessage::Hit {
            info: DamageInfo { filter: DamageFilter::Player, .. }, ..
        } = m {
            vec![]
        } else {
            vec![
                GameAction::Delete,
                GameAction::AddObjects(
                    vec![
                        box Explosion::with_bounds(
                            &mut ctx.renderer,
                            time.total,
                            BoundingRect {
                                x: self.bounds.x,
                                y: self.bounds.y,
                                width: 10.0,
                                height: 10.0,
                            }
                        )
                    ]
                ),
            ]
        }
    }
}

pub struct Bullet {
    pub bounds: BoundingRect,
    pub velocity: [f64; 2],
}

impl Bullet {
    pub fn new([x, y]: [f64; 2]) -> Bullet {
        Bullet {
            bounds: BoundingRect {
                x: x,
                y: y,
                width: 8.0,
                height: 4.0,
            },
            velocity: [1800.0, 0.0],
        }
    }
}

impl GameObject<Keys, Texture> for Bullet {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        time: GameTime
    ) -> Vec<GameAction<Keys, Texture>> {
        let elapsed = time.elapsed.exact_seconds();

        self.bounds.x += self.velocity[0] * elapsed;
        self.bounds.y += self.velocity[1] * elapsed;

        let screen = context.renderer.output_size().map(
            |(w, h)| BoundingRect::default().with_size(w as _, h as _)
        ).unwrap();

        if self.bounds.intersects(&screen) {
            vec![]
        } else {
            vec![GameAction::Delete]
        }
    }

    fn sprites(&self, _: GameTime)
        -> Vec<(VisibleComponent<Texture>, Dest)>
    {
        vec![
            (
                VisibleRect(Color::RGB(230, 230, 30)).into(),
                self.bounds.try_into().unwrap(),
            )
        ]
    }

    fn bounds(&self) -> Option<Bounds> { Some(self.bounds.into()) }

    fn receive_message<'a>(
        &'a mut self,
        ctx: &mut Context<Keys>,
        time: GameTime,
        m: GameMessage<'a, Keys, Texture>
    ) -> Vec<GameAction<Keys, Texture>> {
        if let GameMessage::Hit {
            info: DamageInfo { filter: DamageFilter::Player, .. },
            ..
        } = m {
            vec![]
        } else {
            vec![
                GameAction::Delete,
                GameAction::AddObjects(
                    vec![
                        box Explosion::with_bounds(
                            &mut ctx.renderer,
                            time.total,
                            BoundingRect {
                                x: self.bounds.x,
                                y: self.bounds.y,
                                width: 10.0,
                                height: 10.0,
                            }
                        )
                    ]
                ),
            ]
        }
    }

    fn on_hit(&self) -> Option<GameMessage<Keys, Texture>> {
        Some(
            GameMessage::Hit {
                other: self as _,
                info: DamageInfo {
                    damage: 20,
                    filter: DamageFilter::Player,
                },
            }
        )
    }
}
