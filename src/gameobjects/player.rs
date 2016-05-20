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
use std::collections::HashMap;
use sdl2::pixels::Color;
use sdl2::render::{Texture, Renderer};
use sdl2_image::LoadTexture;

const ASTEROID_PATH: &'static str = "assets/asteroid.png";
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

pub enum GameAction<K: KeySet, T: GetSize> {
    Delete,
    AddObjects(Vec<Box<GameObject<K, T>>>),
}

pub trait GameObject<K: KeySet, T: GetSize> {
    fn update(
        &mut self,
        context: &mut Context<K>,
        time: GameTime
    ) -> Option<GameAction<K, T>>;
    fn sprites(&self, time: GameTime) -> Vec<(VisibleComponent<Texture>, Dest)>;
    fn bounds(&self) -> &Bounds;
}

pub struct Ship {
    pub bounds: Bounds,
    pub dpos: [f64; 2],
    pub sprites: HashMap<ShipFrame, Sprite<Texture>>,
}

impl Ship {
    fn spawn_bullets(&self) -> Vec<Box<GameObject<Keys, Texture>>> {
        let cannons_x = self.bounds.x + 30.0;
        let cannon1_y = self.bounds.y + 6.0;
        let cannon2_y = self.bounds.y + self.bounds.height - 10.0;

        vec![
            box Bullet::new([cannons_x, cannon1_y]),
            box Bullet::new([cannons_x, cannon2_y]),
        ]
    }
}

impl GameObject<Keys, Texture> for Ship {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        time: GameTime
    ) -> Option<GameAction<Keys, Texture>> {
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

        if context.events.pressed.space {
            Some(GameAction::AddObjects(self.spawn_bullets()))
        } else {
            None
        }
    }

    fn sprites(&self, time: GameTime)
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

    fn bounds(&self) -> &Bounds { &self.bounds }
}

pub struct Asteroid {
    pub sprite: AnimatedSprite<u32, Texture>,
    pub bounds: Bounds,
    pub velocity: [f64; 2],
}

impl GameObject<Keys, Texture> for Asteroid {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        time: GameTime
    ) -> Option<GameAction<Keys, Texture>> {
        let elapsed = time.elapsed.exact_seconds();

        self.bounds.x += self.velocity[0] * elapsed;
        self.bounds.y += self.velocity[1] * elapsed;

        None
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

    fn bounds(&self) -> &Bounds { &self.bounds }
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
            velocity: [500.0, 0.0],
        }
    }
}

impl GameObject<Keys, Texture> for Bullet {
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        time: GameTime
    ) -> Option<GameAction<Keys, Texture>> {
        let elapsed = time.elapsed.exact_seconds();

        self.bounds.x += self.velocity[0] * elapsed;
        self.bounds.y += self.velocity[1] * elapsed;

        let screen = context.renderer.output_size().map(
            |(w, h)| Bounds::default().with_size(w as _, h as _)
        ).unwrap();

        if self.bounds.intersects(&screen) {
            None
        } else {
            Some(GameAction::Delete)
        }
    }

    fn sprites(&self, time: GameTime)
        -> Vec<(VisibleComponent<Texture>, Dest)>
    {
        vec![
            (
                VisibleRect(Color::RGB(230, 230, 30)).into(),
                self.bounds.try_into().unwrap(),
            )
        ]
    }

    fn bounds(&self) -> &Bounds { &self.bounds }
}

pub struct ShipView {
    player: Ship,
    asteroids: Vec<Asteroid>,
    bullets: Vec<Box<GameObject<Keys, Texture>>>,
    background: Background,
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
        let asteroids = vec![
            Asteroid::new(
                renderer,
                0,
                [400.0, 20.0]
            ),
            Asteroid::new(
                renderer,
                0,
                [400.0, 130.0]
            ),
            Asteroid::new(
                renderer,
                0,
                [400.0, 240.0]
            ),
        ];

        ShipView {
            player: Ship {
                bounds: Bounds {
                    width: 50.0,
                    height: 50.0,
                    x: 0.0,
                    y: 0.0,
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
            },
            asteroids: asteroids,
            bullets: vec![],
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

pub struct ShipViewBuilder;

impl ViewBuilder<Keys> for ShipViewBuilder {
    fn build_view(self: Box<Self>, context: &mut Context<Keys>)
        -> Box<View<Keys>>
    {
        Box::new(Some(ShipView::new(context.renderer)))
    }
}

impl View<Keys> for Option<ShipView> {
    fn render(
        &mut self,
        context: &mut Context<Keys>,
        elapsed: u32
    ) -> Option<Action<Keys>> {
        if context.events.down.escape {
            let slf = if let Some(ship_view) = self.take() {
                ship_view
            } else {
                unreachable!()
            };

            return Some(
                Action::ChangeView(
                    box PauseMenuBuilder(box Some(slf) as Box<View<_>>)
                )
            );
        }

        if let &mut Some(ref mut slf) = self {
            slf.render(context, elapsed)
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
        use std::convert::TryInto;

        self.total_time += elapsed;

        let dt = elapsed.exact_seconds();

        let game_time = GameTime {
            elapsed: elapsed,
            total: self.total_time,
        };

        if context.events.down.quit {
            return Some(Action::Quit);
        }

        let (sw, sh) = context.renderer.output_size().map(
            |(a, b)| (a as f64, b as f64)
        ).unwrap();

        for asteroid in &mut self.asteroids {
            asteroid.update(context, game_time);
        }

        let bullets = self.bullets.drain(..).filter_map(
            |mut bullet| if let Some(GameAction::Delete) = bullet.update(
                context,
                game_time
            ) {
                None
            } else {
                Some(bullet)
            }
        ).collect();

        self.bullets = bullets;

        match self.player.update(context, game_time) {
            Some(GameAction::AddObjects(v)) =>
                self.bullets.extend(v),
            _ => {},
        }

        context.renderer.set_draw_color(Color::RGB(0, 0, 0));
        context.renderer.clear();

        let (screen_w, screen_h) = context.renderer.output_size().unwrap();

        let screen = Dest::default().with_size(screen_w, screen_h);

        for (sprite, dest) in self.background.get_destinations(
            screen,
            self.total_time
        ).into_iter().map(|(s, d)| (s.into(), d)).chain(
            self.player.sprites(game_time).into_iter()
        ).chain(
            self.asteroids.iter().flat_map(|a| a.sprites(game_time).into_iter())
        ).chain(
            self.bullets.iter().flat_map(|a| a.sprites(game_time).into_iter())
        ) {
            context.renderer.copy_renderable(
                &sprite,
                dest
            );
        }

        None
    }
}
