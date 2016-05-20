use ::lazy::Lazy;
use ::time::TimeExtensions;
use ::coalesce::Coalesce;
use ::events::Keys;
use ::view::{Context, View, ViewBuilder, Action};
use ::graphics::sprites::{LoadSprite, CopyRenderable, Sprite};
use ::graphics::font_cache::FontCache;
use ::gameobjects::background::ParallaxSet;
use ::gameobjects::player::*;
use ::gameobjects::Dest;

use std::marker::PhantomData;
use std::boxed::FnBox;
use sdl2_ttf::Font;
use sdl2::render::{Texture, Renderer};
use sdl2::pixels::Color;

pub type Background = [([f64; 2], [f64; 2], Sprite<Texture>); 3];

pub struct Menu<
    Time: TimeExtensions,
    I,
    B,
    T,
    F: FnOnce() -> T = Box<FnBox() -> T>
> where
    for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<T, F>>,
    for<'a> &'a B: IntoIterator<Item=&'a ([f64; 2], [f64; 2], Sprite<Texture>)>
{
    pub items: I,
    background: ParallaxSet<Time, Texture, B>,
    total_time: u32,
    count: usize,
    selected: usize,
    _phantom_v: PhantomData<T>,
    _phantom_f: PhantomData<F>,
}

impl<I, B, F: FnOnce() -> Action<Keys>>
    View<Keys> for Menu<u32, I, B, Action<Keys>, F>
    where
        for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<Action<Keys>, F>>,
        for<'a> &'a B: IntoIterator<Item=&'a ([f64; 2], [f64; 2], Sprite<Texture>)>
{
    fn render(
        &mut self,
        context: &mut Context<Keys>,
        elapsed: u32
    ) -> Option<Action<Keys>> {
        context.renderer.set_draw_color(Color::RGB(0, 0, 0));
        context.renderer.clear();

        self.total_time += elapsed;

        let (screen_w, screen_h) = context.renderer.output_size().unwrap();

        let y_gutter = 70;
        let y_offset = (screen_h as usize - y_gutter * self.count) / 2;

        let screen = Dest::default().with_size(screen_w, screen_h);

        for (sprite, dest) in self.background.get_destinations(
            screen,
            self.total_time
        ) {
            context.renderer.copy_renderable(
                &sprite,
                dest
            );
        }

        for (i, item) in self.items.into_iter().enumerate() {
            let sprite = if i == self.selected {
                if context.events.down.space {
                    return item.on_select.take().map(|a| a.consume());
                }

                &item.hover_sprite
            } else {
                &item.idle_sprite
            };

            context.renderer.copy_renderable(
                sprite,
                Dest {
                    x: ((screen_w - sprite.mask.width) / 2) as _,
                    y:
                        (y_offset + y_gutter * i) as i32 -
                        (sprite.mask.height / 2) as i32,
                    width: sprite.mask.width,
                    height: sprite.mask.height,
                }
            );
        }

        self.selected = (
            if context.events.pressed.down {
                self.selected + 1
            } else if context.events.pressed.up {
                (self.selected + self.count) - 1
            } else {
                self.selected
            }
        ) % self.count;

        None
    }
}

impl<Time: TimeExtensions + Copy + Default, I, T, F: FnOnce() -> T>
    Menu<Time, I, [([f64; 2], [f64; 2], Sprite<Texture>); 3], T, F>
    where
        for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<T, F>>,
{
    pub fn new(renderer: &mut Renderer, mut items: I) -> Self {
        let count = (&mut items).into_iter().count();

        Menu {
            items: items,
            background: ParallaxSet::new(
                [
                    (
                        [-200.0, 0.0],
                        [0.0, 0.0],
                        renderer.load_sprite(
                            "assets/spaceBG.png"
                        ).unwrap()
                    ),
                    (
                        [-400.0, 0.0],
                        [0.0, 30.0],
                        renderer.load_sprite(
                            "assets/spaceFG.png"
                        ).unwrap()
                    ),
                    (
                        [-500.0, 0.0],
                        [0.0, 0.0],
                        renderer.load_sprite(
                            "assets/spaceFG.png"
                        ).unwrap()
                    ),
                ],
                Default::default()
            ),
            total_time: 0,
            count: count,
            selected: 0,
            _phantom_v: PhantomData,
            _phantom_f: PhantomData,
        }
    }
}

pub struct MenuItem<T, F: FnOnce() -> T = Box<FnBox() -> T>> {
    idle_sprite: Sprite<Texture>,
    hover_sprite: Sprite<Texture>,
    on_select: Option<Lazy<T, F>>,
}

fn get_sprites(
    renderer: &mut Renderer,
    text: &str,
    fonts: (&Font, &Font)
) -> (Sprite<Texture>, Sprite<Texture>) {
    let (idle_color, hover_color) = (
        Color::RGB(120, 120, 120),
        Color::RGB(255, 255, 255),
    );

    let color_to_sprite = |color, font: &Font|
        font.render(text).blended(color).ok()
            .and_then(
                |surface| renderer.create_texture_from_surface(&surface).ok()
            ).map(Sprite::new);

    (
        color_to_sprite(idle_color, fonts.0),
        color_to_sprite(hover_color, fonts.1),
    ).coalesce().unwrap()
}

impl<T, F: FnOnce() -> T> MenuItem<T, F> {
    pub fn from_function(
        idle: Sprite<Texture>,
        hover: Sprite<Texture>,
        f: F
    ) -> Self {
        MenuItem {
            idle_sprite: idle,
            hover_sprite: hover,
            on_select: Some(Lazy::Function(f)),
        }
    }

    pub fn from_value(
        idle: Sprite<Texture>,
        hover: Sprite<Texture>,
        v: T
    ) -> Self {
        MenuItem {
            idle_sprite: idle,
            hover_sprite: hover,
            on_select: Some(Lazy::Value(v)),
        }
    }
}

pub struct MainMenuBuilder;

impl ViewBuilder<Keys> for MainMenuBuilder {
    fn build_view(self: Box<Self>, context: &mut Context<Keys>)
        -> Box<View<Keys>>
    {
        Box::new(
            main_menu(context.renderer, context.font_cache, box ShipViewBuilder)
        )
    }
}

pub struct PauseMenuBuilder<T>(pub Box<View<T>>);

impl ViewBuilder<Keys> for PauseMenuBuilder<Keys> {
    fn build_view(self: Box<Self>, context: &mut Context<Keys>)
        -> Box<View<Keys>>
    {
        let next = self.0;

        Box::new(
            main_menu(
                context.renderer,
                context.font_cache,
                box move |_: &mut Context<Keys>| next
            )
        )
    }
}

pub fn main_menu(
    renderer: &mut Renderer,
    cache: &mut FontCache,
    view: Box<ViewBuilder<Keys>>
) -> Menu<
    u32,
    [MenuItem<Action<Keys>>; 2],
    Background,
    Action<Keys>
> {
    let path = "assets/belligerent.ttf";
    let cache = cache.with_loaded(path, 38)
        .and_then(|c| c.with_loaded(path, 32))
        .unwrap();

    let font0 = cache.get(path, 32).unwrap();
    let font1 = cache.get(path, 38).unwrap();
    let fonts = (font0, font1);

    let items = [
        {
            let (idle, hover) = get_sprites(renderer, "Play", fonts);
            MenuItem::from_value(
                idle,
                hover,
                Action::ChangeView(view)
            )
        },
        {
            let (idle, hover) = get_sprites(renderer, "Quit", fonts);
            MenuItem::from_value(idle, hover, Action::Quit)
        },
    ];

    Menu::new(renderer, items)
}
