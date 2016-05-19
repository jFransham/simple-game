use ::gameobjects::*;
use ::gameobjects::background::*;
use ::events::*;
use ::view::*;
use ::graphics::sprites::{build_spritesheet, LoadSprite, Sprite, CopySprite};
use ::set::Set;
use ::time::*;

use std::collections::HashMap;
use sdl2::pixels::Color;
use sdl2::render::{Texture, Renderer};
use sdl2_image::LoadTexture;

type Background = ParallaxSet<u32, Texture, [([f64; 2], Sprite<Texture>); 3]>;

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

pub struct Ship {
    pub bounds: Bounds,
    pub sprites: HashMap<ShipFrame, Sprite<Texture>>,
}

pub struct ShipView {
    player: Ship,
    background: Background,
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
        ShipView {
            player: Ship {
                bounds: Bounds {
                    width: 50.0,
                    height: 50.0,
                    x: 0.0,
                    y: 0.0,
                },
                sprites: ALL_FRAMES.into_iter()
                    .cloned()
                    .zip(
                        build_spritesheet(
                            renderer.load_texture(
                                "assets/spaceship.png".as_ref()
                            ).unwrap(),
                            43,
                            39
                        ).into_iter()
                    ).collect(),
            },
            background: Background::new(
                [
                    (
                        [-20.0, 0.0],
                        renderer.load_sprite(
                            "assets/spaceBG.png"
                        ).unwrap()
                    ),
                    (
                        [-40.0, 0.0],
                        renderer.load_sprite(
                            "assets/spaceMG.png"
                        ).unwrap()
                    ),
                    (
                        [-80.0, 0.0],
                        renderer.load_sprite(
                            "assets/spaceFG.png"
                        ).unwrap()
                    ),
                ],
                0
            ),
        }
    }
}

impl View<Keys> for ShipView {
    fn render(&mut self, context: Context<Keys>) -> Option<Action<Keys>> {
        use std::convert::TryInto;

        let player_speed = 230.0;

        let keys = context.events.down;

        if keys.quit || keys.escape {
            return Some(Action::Quit)
        }

        let [dx, dy] = get_control(
            keys.up,
            keys.down,
            keys.left,
            keys.right,
        );

        let elapsed = context.time.elapsed.exact_seconds();

        self.player.bounds.x += dx * elapsed * player_speed;
        self.player.bounds.y += dy * elapsed * player_speed;

        let (sw, sh) = context.renderer.output_size().map(
            |(a, b)| (a as f64, b as f64)
        ).unwrap();

        self.player.bounds = self.player.bounds.move_inside(
            &Bounds {
                width: sw,
                height: sh,
                .. Default::default()
            }
        ).unwrap();

        context.renderer.set_draw_color(Color::RGB(0, 0, 0));
        context.renderer.clear();

        {
            let (screen_w, screen_h) = context.renderer.output_size().unwrap();

            let screen = Dest::default().with_size(screen_w, screen_h);

            for (sprite, [offset_x, offset_y]) in self.background.offset_sprites(
                context.time.total
            ) {
                use itertools::*;
                let (w, h) = (sprite.mask.width, sprite.mask.height);

                for dest in (offset_x..screen_w as i32).step_by(w as i32)
                    .cartesian_product(
                        (offset_y..screen_h as i32).step_by(h as i32)
                    )
                    .map(|(x, y)|
                        Dest {
                            x: x,
                            y: y,
                            width: sprite.mask.width,
                            height: sprite.mask.height,
                        }
                    )
                    .filter(|c| screen.intersects(c))
                {
                    context.renderer.copy_sprite(
                        &sprite,
                        dest
                    );
                }
            }
        }

        let sprite = &self.player.sprites[&get_frame([dx, dy])];

        context.renderer.copy_sprite(
            sprite,
            self.player.bounds.try_into().unwrap()
        );

        None
    }
}

