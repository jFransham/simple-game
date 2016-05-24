use ::gameobjects::*;
use ::gameobjects::main_menu::PauseMenuBuilder;
use ::gameobjects::background::*;
use ::events::*;
use ::view::*;
use ::graphics::sprites::{
    build_spritesheet,
    AnimatedSprite,
    VisibleComponent,
    VisibleRect,
    LoadSprite,
    Sprite,
    CopyRenderable,
    GetSize,
};
use ::time::*;
use ::set::Set;

use std::convert::TryInto;
use std::ops::Index;
use std::collections::HashMap;
use sdl2::pixels::Color;
use sdl2::render::{Texture, Renderer};
use sdl2_image::LoadTexture;

const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const EXPLOSION_PATH: &'static str = "assets/explosion.png";
const SHIP_PATH: &'static str = "assets/spaceship.png";
const BACKGROUND_PATHS: [&'static str; 3] = [
    "assets/spaceBG.png",
    "assets/spaceMG.png",
    "assets/spaceFG.png",
];

type Background = ParallaxSet<
    u32,
    Texture,
    [([f64; 2], [f64; 2], Sprite<Texture>); 3]
>;

pub enum DamageFilter {
    Player,
    Enemy,
}

pub struct DamageInfo {
    filter: DamageFilter,
    damage: u32,
}

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

static ALL_FRAMES: [ShipFrame; 9] = [
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

type EntityRef = usize;

pub struct Entities<'a, K: KeySet + 'a, T: GetSize + 'a> {
    before: &'a [Box<GameObject<K, T>>],
    after: &'a [Box<GameObject<K, T>>],
}

impl<'a, K: KeySet + 'a, T: GetSize + 'a> Index<usize> for Entities<'a, K, T> {
    type Output = Box<GameObject<K, T>>;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.before.len() {
            &self.before[index]
        } else {
            &self.after[index]
        }
    }
}

pub enum GameAction<K: KeySet, T: GetSize> {
    Delete,
    AddObjects(Vec<Box<GameObject<K, T>>>),
}

pub enum GameMessage<'a, K: KeySet + 'a, T: GetSize + 'a> {
    Hit {
        other: &'a (GameObject<K, T> + 'a),
        info: DamageInfo,
    },
    __Other,
}

pub trait GameObject<K: KeySet, T: GetSize> {
    fn update(
        &mut self,
        context: &mut Context<K>,
        time: GameTime
    ) -> Vec<GameAction<K, T>>;
    fn sprites(&self, time: GameTime) -> Vec<(VisibleComponent<Texture>, Dest)>;
    fn bounds(&self) -> Option<&Bounds> { None }

    fn receive_message<'a>(
        &'a mut self,
        _: &mut Context<K>,
        _: GameTime,
        _: GameMessage<'a, K, T>
    ) -> Vec<GameAction<Keys, Texture>> {
        vec![]
    }

    fn on_hit(&self) -> Option<GameMessage<K, T>> {
        None
    }
}

pub type SimpleObject = Box<GameObject<Keys, Texture>>;

pub struct GunArgs<'a> {
    pub bounds: &'a Bounds,
}

pub trait Gun: Sized {
    fn spawn_bullets<'a>(
        &'a mut self,
        args: GunArgs<'a>,
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
    fn spawn_bullets<'a>(
        &mut self,
        args: GunArgs<'a>,
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

pub struct Ship<G: Gun> {
    pub bounds: Bounds,
    pub gun: G,
    pub dpos: [f64; 2],
    pub sprites: HashMap<ShipFrame, Sprite<Texture>>,
}

impl<G: Gun> GameObject<Keys, Texture> for Ship<G> {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        time: GameTime
    ) -> Vec<GameAction<Keys, Texture>> {
        let player_speed = 230.0;

        let dt = time.elapsed.exact_seconds();

        let (sw, sh) = context.renderer.output_size().map(
            |(a, b)| (a as f64, b as f64)
        ).unwrap();

        let [dx, dy] = {
            let keys = &context.events.down;

            get_control(
                keys.up,
                keys.down,
                keys.left,
                keys.right,
            )
        };

        self.dpos = [dx, dy];

        self.bounds.x += self.dpos[0] * dt * player_speed;
        self.bounds.y += self.dpos[1] * dt * player_speed;

        self.bounds = self.bounds.move_inside(
            &Bounds {
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
                        GunArgs { bounds: &self.bounds, },
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
                self.sprites[
                    &get_frame(
                        [self.dpos[0], self.dpos[1]]
                    )
                ].clone().into(),
                self.bounds.try_into().unwrap(),
            )
        ]
    }

    fn bounds(&self) -> Option<&Bounds> { Some(&self.bounds) }
}

pub struct Explosion {
    pub sprite: AnimatedSprite<u32, Texture>,
    pub position: [f64; 2],
}

impl Explosion {
    pub fn new(renderer: &mut Renderer, now: u32, pos: [f64; 2]) -> Self {
        Explosion {
            sprite: AnimatedSprite::from_spritesheet(
                now,
                40.0,
                {
                    let mut sprites = build_spritesheet(
                        renderer.load_texture(
                            EXPLOSION_PATH.as_ref()
                        ).unwrap(),
                        96,
                        96
                    );

                    for _ in 0..3 { sprites.pop(); }

                    sprites
                }
            ),
            position: pos,
        }
    }
}

impl GameObject<Keys, Texture> for Explosion {
    fn update(
        &mut self,
        _: &mut Context<Keys>,
        time: GameTime,
    ) -> Vec<GameAction<Keys, Texture>> {
        if self.sprite.completed(time.total) {
            vec![GameAction::Delete]
        } else {
            vec![]
        }
    }

    fn sprites(&self, time: GameTime)
        -> Vec<(VisibleComponent<Texture>, Dest)>
    {
        let sprite = self.sprite.frame(time.total).clone();
        let (w, h) = (sprite.mask.width, sprite.mask.height);

        vec![
            (
                sprite.into(),
                Dest {
                    x: self.position[0] as _,
                    y: self.position[1] as _,
                    width: w,
                    height: h,
                }
            )
        ]
    }
}

pub struct Asteroid {
    pub sprite: AnimatedSprite<u32, Texture>,
    pub hp: u32,
    pub bounds: Bounds,
    pub velocity: [f64; 2],
}

impl GameObject<Keys, Texture> for Asteroid {
    fn update(
        &mut self,
        _: &mut Context<Keys>,
        time: GameTime,
    ) -> Vec<GameAction<Keys, Texture>> {
        let elapsed = time.elapsed.exact_seconds();

        self.bounds.x += self.velocity[0] * elapsed;
        self.bounds.y += self.velocity[1] * elapsed;

        vec![]
    }

    fn sprites(&self, time: GameTime)
        -> Vec<(VisibleComponent<Texture>, Dest)>
    {
        vec![
            (
                self.sprite.frame(time.total).clone().into(),
                self.bounds.try_into().unwrap(),
            )
        ]
    }

    fn receive_message<'a>(
        &'a mut self,
        ctx: &mut Context<Keys>,
        time: GameTime,
        msg: GameMessage<'a, Keys, Texture>
    ) -> Vec<GameAction<Keys, Texture>> {
        if let GameMessage::Hit { info: DamageInfo { damage, .. }, .. } = msg {
            if damage > self.hp {
                vec![
                    GameAction::AddObjects(
                        vec![
                            box Explosion::new(
                                &mut ctx.renderer,
                                time.total,
                                [self.bounds.x, self.bounds.y]
                            ),
                        ]
                    ),
                    GameAction::Delete,
                ]
            } else {
                self.hp -= damage;

                vec![]
            }
        } else {
            vec![]
        }
    }

    fn on_hit(&self) -> Option<GameMessage<Keys, Texture>> {
        Some(
            GameMessage::Hit {
                other: self as _,
                info: DamageInfo {
                    damage: 0,
                    filter: DamageFilter::Enemy,
                },
            }
        )
    }

    fn bounds(&self) -> Option<&Bounds> { Some(&self.bounds) }
}

impl Asteroid {
    pub fn new(
        renderer: &mut Renderer,
        now: u32,
        [x, y]: [f64; 2]
    ) -> Asteroid {
        let [w, h] = [96; 2];

        Asteroid {
            sprite: AnimatedSprite::from_spritesheet(
                now,
                30.0,
                {
                    let mut sprites = build_spritesheet(
                        renderer.load_texture(
                            ASTEROID_PATH.as_ref()
                        ).unwrap(),
                        w,
                        h
                    );

                    for _ in 0..4 { sprites.pop(); }

                    sprites
                }
            ),
            hp: 100,
            bounds: Bounds {
                x: x,
                y: y,
                width: w as _,
                height: h as _,
            },
            velocity: [-50.0, 0.0],
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum BulletKind {
    Standard,
    Sine,
}

pub struct SineBullet {
    pub bounds: Bounds,
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
            bounds: Bounds {
                x: x,
                y: y,
                width: 8.0,
                height: 4.0,
            },
            origin_y: y,
        }
    }
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
    fn spawn_bullets<'a>(
        &mut self,
        args: GunArgs<'a>,
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

        let cannons_x = args.bounds.x + 30.0;
        let cannon1_y = args.bounds.y + 6.0;
        let cannon2_y = args.bounds.y + args.bounds.height - 10.0;

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
    fn spawn_bullets<'a>(
        &mut self,
        args: GunArgs<'a>,
        time: GameTime
    ) -> Vec<SimpleObject> {
        let time_diff = time.total - self.last_shot_at;

        if time_diff < 400 {
            return vec![];
        }

        self.last_shot_at = time.total;

        let cannons_x = args.bounds.x + 30.0;
        let cannon1_y = args.bounds.y + 6.0;
        let cannon2_y = args.bounds.y + args.bounds.height - 10.0;

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
            |(w, h)| Bounds::default().with_size(w as _, h as _)
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

    fn bounds(&self) -> Option<&Bounds> { Some(&self.bounds) }

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
        _: &mut Context<Keys>,
        _: GameTime,
        m: GameMessage<'a, Keys, Texture>
    ) -> Vec<GameAction<Keys, Texture>> {
        if let GameMessage::Hit {
            info: DamageInfo { filter: DamageFilter::Player, .. }, ..
        } = m {
            vec![]
        } else {
            vec![GameAction::Delete]
        }
    }
}

pub struct Bullet {
    pub bounds: Bounds,
    pub velocity: [f64; 2],
}

impl Bullet {
    pub fn new([x, y]: [f64; 2]) -> Bullet {
        Bullet {
            bounds: Bounds {
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
            |(w, h)| Bounds::default().with_size(w as _, h as _)
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

    fn bounds(&self) -> Option<&Bounds> { Some(&self.bounds) }

    fn receive_message<'a>(
        &'a mut self,
        _: &mut Context<Keys>,
        _: GameTime,
        m: GameMessage<'a, Keys, Texture>
    ) -> Vec<GameAction<Keys, Texture>> {
        if let GameMessage::Hit {
            info: DamageInfo { filter: DamageFilter::Player, .. }, ..
        } = m {
            vec![]
        } else {
            vec![GameAction::Delete]
        }
    }

    fn on_hit<'a>(&'a self) -> Option<GameMessage<'a, Keys, Texture>> {
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

pub struct ShipViewBuilder;

#[allow(boxed_local)]
impl ViewBuilder<Keys> for ShipViewBuilder {
    fn build_view(self: Box<Self>, context: &mut Context<Keys>)
        -> Box<View<Keys>>
    {
        Box::new(Some(ShipView::new(context.renderer)))
    }
}

pub struct ShipView {
    objects: Vec<SimpleObject>,
    background: Background,
    last_asteroid_time: u32,
    total_time: u32,
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

fn get_control(up: bool, down: bool, left: bool, right: bool) -> [f64; 2] {
    let x_request =
        match (left, right) {
            (true, true) | (false, false) => 0.0,
            (true, _) => -1.0,
            (_, true) => 1.0,
        };

    let y_request =
        match (up, down) {
            (true, true) | (false, false) => 0.0,
            (true, _) => -1.0,
            (_, true) => 1.0,
        };

    normalize([x_request, y_request])
}

#[allow(collapsible_if)]
fn get_frame([x, y]: [f64; 2]) -> ShipFrame {
    use self::ShipFrame::*;

    if x < 0.0 {
        if y < 0.0 {
            UpSlow
        } else if y > 0.0 {
            DownSlow
        } else {
            MidSlow
        }
    } else if x > 0.0 {
        if y < 0.0 {
            UpFast
        } else if y > 0.0 {
            DownFast
        } else {
            MidFast
        }
    } else {
        if y < 0.0 {
            UpNorm
        } else if y > 0.0 {
            DownNorm
        } else {
            MidNorm
        }
    }
}

impl ShipView {
    pub fn new(renderer: &mut Renderer) -> Self {
        let (_, screen_h) = renderer.output_size().unwrap();

        ShipView {
            last_asteroid_time: 0,
            objects: vec![
                box Ship {
                    gun: ShipGun {
                        kind: BulletKind::Standard,
                        standard: StandardGun::new(0),
                        sine: SineGun::new(0),
                    },
                    bounds: Bounds {
                        width: 50.0,
                        height: 50.0,
                        x: 0.0,
                        y: (screen_h / 2) as f64 - 25.0,
                    },
                    dpos: [0.0, 0.0],
                    sprites: ALL_FRAMES.into_iter()
                        .cloned()
                        .zip(
                            build_spritesheet(
                                renderer.load_texture(
                                    SHIP_PATH.as_ref()
                                ).unwrap(),
                                43,
                                39
                            ).into_iter()
                        ).collect(),
                }
            ],
            background: Background::new(
                [
                    (
                        [-20.0, 0.0],
                        [0.0, 0.0],
                        renderer.load_sprite(
                            BACKGROUND_PATHS[0]
                        ).unwrap()
                    ),
                    (
                        [-40.0, 0.0],
                        [0.0, 0.0],
                        renderer.load_sprite(
                            BACKGROUND_PATHS[1]
                        ).unwrap()
                    ),
                    (
                        [-80.0, 0.0],
                        [0.0, 0.0],
                        renderer.load_sprite(
                            BACKGROUND_PATHS[2]
                        ).unwrap()
                    ),
                ],
                0
            ),
            total_time: 0,
        }
    }
}

impl View<Keys> for Option<ShipView> {
    fn render(
        &mut self,
        context: &mut Context<Keys>,
        _: u32
    ) -> Option<Action<Keys>> {
        if context.events.down.escape {
            let slf = if let Some(ship_view) = self.take() {
                ship_view
            } else {
                return Some(Action::ChangeView(box ShipViewBuilder));
            };

            return Some(
                Action::ChangeView(
                    box PauseMenuBuilder(box Some(slf) as Box<View<_>>)
                )
            );
        }

        if let Some(ref mut slf) = *self {
            slf.render(context, 10)
        } else {
            None
        }
    }
}

impl View<Keys> for ShipView {
    fn render(
        &mut self,
        context: &mut Context<Keys>,
        elapsed: u32
    ) -> Option<Action<Keys>> {
        use ::split_iterator::*;
        use ::coalesce::*;
        use std::mem;
        use rand::random;

        self.total_time += elapsed;

        let (screen_w, screen_h) = context.renderer.output_size().unwrap();

        let asteroid_interval = 1000;

        if self.total_time - self.last_asteroid_time > asteroid_interval {
            use std::num::Wrapping;

            self.objects.push(
                box Asteroid::new(
                    context.renderer,
                    self.total_time,
                    [
                        screen_w as _,
                        (random::<u32>() % (screen_h - 96)) as _
                    ]
                ) as _
            );

            self.last_asteroid_time = self.total_time;
        }

        let game_time = GameTime {
            elapsed: elapsed,
            total: self.total_time,
        };

        if context.events.down.quit {
            return Some(Action::Quit);
        }

        context.renderer.set_draw_color(Color::RGB(0, 0, 0));
        context.renderer.clear();

        let (screen_w, screen_h) = context.renderer.output_size().unwrap();

        let screen = Dest::default().with_size(screen_w, screen_h);

        let messages: Vec<Vec<GameAction<Keys, Texture>>> = {
            let mut message_container = self.objects.iter_mut().map(
                |m| { let update = m.update(context, game_time); (m, update) }
            ).collect::<Vec<_>>();

            message_container.split_iter_mut().map(
                |(&mut (ref mut head, ref mut msgs), ref mut tail)| {
                    let bounds = head.bounds().cloned();

                    mem::replace(msgs, vec![]).into_iter().chain(
                        tail.iter_mut().flat_map(
                            |&mut (ref mut other, ref mut other_msgs)|
                                if (bounds, other.bounds()).coalesce()
                                    .map_or(
                                        false,
                                        |(a, b)| a.intersects(b)
                                    )
                                {
                                    if let Some(msg) = head.on_hit() {
                                        other_msgs.extend(
                                            other.receive_message(
                                                context,
                                                game_time,
                                                msg
                                            )
                                        );
                                    }

                                    if let Some(msg) = other.on_hit() {
                                        head.receive_message(
                                            context,
                                            game_time,
                                            msg
                                        )
                                    } else {
                                        vec![]
                                    }
                                } else {
                                    vec![]
                                }
                        )
                    ).collect()
                }
            ).collect::<Vec<_>>()
        };

        self.objects = mem::replace(&mut self.objects, vec![]).into_iter()
            .zip(messages.into_iter())
            .flat_map(
                |(obj, obj_msgs)| {
                    let maybe_obj = if obj_msgs.iter().any(
                        |m| if let &GameAction::Delete = m {
                            true
                        } else {
                            false
                        }
                    ) {
                        None
                    } else {
                        Some(obj)
                    };

                    obj_msgs.into_iter().flat_map(
                        |msg| if let GameAction::AddObjects(objs) = msg {
                            objs
                        } else {
                            vec![]
                        }
                    ).chain(maybe_obj.into_iter())
                }
            ).collect();

        for (sprite, dest) in self.background.get_destinations(
            screen,
            self.total_time
        ).into_iter().map(|(s, d)| (s.into(), d)).chain(
            self.objects.iter().flat_map(|a| a.sprites(game_time).into_iter())
        ) {
            context.renderer.copy_renderable(
                &sprite,
                dest
            );
        }

        None
    }
}
