use ::gameobjects::*;
use ::gameobjects::main_menu::PauseMenuBuilder;
use ::gameobjects::background::*;
use ::events::*;
use ::view::*;
use ::graphics::sprites::{
    build_spritesheet,
    AnimatedSprite,
    VisibleComponent,
    LoadSprite,
    Sprite,
    CopyRenderable,
    GetSize,
};
use ::time::*;
use ::set::Intersects;

use std::convert::TryInto;
use sdl2::pixels::Color;
use sdl2::render::{Texture, Renderer};
use sdl2_image::LoadTexture;

mod ship;
pub mod command_builder;

use self::ship::*;

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

pub enum GameAction<K: KeySet, T: GetSize> {
    Delete,
    AddObjects(Vec<Box<GameObject<K, T>>>),
    // Broadcast(Vec<Broadcast>),
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
    fn bounds(&self) -> Option<Bounds> { None }

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

pub struct Explosion {
    pub sprite: AnimatedSprite<u32, Texture>,
    pub bounds: BoundingRect,
}

impl Explosion {
    pub fn new(renderer: &mut Renderer, now: u32, [x, y]: [f64; 2]) -> Self {
        Self::with_bounds(
            renderer,
            now,
            BoundingRect {
                x: x,
                y: y,
                width: 96.0,
                height: 96.0,
            }
        )
    }

    pub fn with_bounds(
        renderer: &mut Renderer,
        now: u32,
        bounds: BoundingRect
    ) -> Self {
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
            bounds: bounds,
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
        vec![
            (
                self.sprite.frame(time.total).clone().into(),
                self.bounds.try_into().unwrap(),
            )
        ]
    }
}

pub struct Asteroid {
    pub sprite: AnimatedSprite<u32, Texture>,
    pub hp: u32,
    pub bounds: BoundingRect,
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

    fn bounds(&self) -> Option<Bounds> {
        let w = self.bounds.width;

        Some(
            Circle {
                x: self.bounds.x + w / 2.0,
                y: self.bounds.y + w / 2.0,
                radius: 40.0,
            }.into()
        )
    }
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
            bounds: BoundingRect {
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

pub struct ShipViewBuilder;

#[allow(boxed_local)]
impl ViewBuilder<Keys, VisibleComponent<Texture>> for ShipViewBuilder {
    fn build_view(self: Box<Self>, context: &mut Context<Keys>)
        -> Box<View<Keys, VisibleComponent<Texture>>>
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
                    bounds: BoundingRect {
                        width: 50.0,
                        height: 50.0,
                        x: 0.0,
                        y: (screen_h / 2) as f64 - 25.0,
                    },
                    dir: Default::default(),
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

impl View<Keys, VisibleComponent<Texture>> for Option<ShipView> {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        _: u32
    ) -> Action<Keys, VisibleComponent<Texture>> {
        if context.events.down.escape {
            let slf = if let Some(ship_view) = self.take() {
                ship_view
            } else {
                return Action::ChangeView(box ShipViewBuilder);
            };

            return Action::ChangeView(
                box PauseMenuBuilder(box Some(slf) as Box<View<_, _>>)
            );
        }

        if let Some(ref mut slf) = *self {
            slf.update(context, 10)
        } else {
            Action::ChangeView(box ShipViewBuilder)
        }
    }
}

impl View<Keys, VisibleComponent<Texture>> for ShipView {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        elapsed: u32
    ) -> Action<Keys, VisibleComponent<Texture>> {
        use ::split_iterator::*;
        use ::coalesce::*;
        use std::mem;
        use rand::random;

        self.total_time += elapsed;

        let (screen_w, screen_h) = context.renderer.output_size().unwrap();

        let asteroid_interval = 1000;

        if self.total_time - self.last_asteroid_time > asteroid_interval {
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
            return Action::Quit;
        }

        context.renderer.set_draw_color(Color::RGB(0, 0, 0));
        context.renderer.clear();

        let (screen_w, screen_h) = context.renderer.output_size().unwrap();

        let screen = Dest::default().with_size(screen_w, screen_h);

        let messages = {
            let mut message_container = self.objects.iter_mut().map(
                |m| { let update = m.update(context, game_time); (m, update) }
            ).collect::<Vec<_>>();

            message_container.split_iter_mut(
                |&mut (ref mut head, ref mut msgs), tail| {
                    let bounds = head.bounds();

                    mem::replace(msgs, vec![]).into_iter().chain(
                        tail.iter_mut().flat_map(
                            |&mut (ref mut other, ref mut other_msgs)|
                                if (bounds, other.bounds()).coalesce()
                                    .map_or(
                                        false,
                                        |(a, b)| a.intersects(&b)
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
                        ).collect::<Vec<_>>()
                    )
                }
            ).collect::<Vec<_>>()
        };

        self.objects = mem::replace(&mut self.objects, vec![]).into_iter()
            .zip(messages)
            .flat_map(
                |(obj, obj_msgs)| {
                    use self::GameAction::*;

                    let (delete, objs): (Vec<_>, Vec<_>) = obj_msgs.into_iter()
                        .map(
                            |m| match m {
                                Delete => (true, vec![]),
                                AddObjects(objs) => (false, objs),
                                // _ => Default::default(),
                            }
                        ).unzip();

                    let maybe_obj = if delete.into_iter().any(|b| b) {
                        None
                    } else {
                        Some(obj)
                    };

                    objs.into_iter().flat_map(
                        |o| o
                    ).chain(maybe_obj.into_iter())
                }
            ).collect();

        Action::Render(
            box self.background.get_destinations(
                screen,
                self.total_time
            ).into_iter().map(|(s, d)| (s.into(), d)).chain(
                self.objects.iter().flat_map(
                    move |a| a.sprites(game_time).into_iter()
                )
            )
        )
    }
}
