use ::lazy::Lazy;
use ::coelesce::Coelesce;
use ::events::Keys;
use ::view::{Context, View, Action};
use ::graphics::sprites::{CopySprite, Sprite};
use ::graphics::font_cache::FontCache;
use ::gameobjects::player::ShipView;
use ::gameobjects::Dest;

use std::marker::PhantomData;
use std::boxed::FnBox;
use sdl2_ttf::{self, Font};
use sdl2::render::{Texture, Renderer};
use sdl2::pixels::Color;

#[derive(Clone)]
pub struct Menu<I, T, F: FnOnce() -> T = Box<FnBox() -> T>>
    where for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<T, F>>
{
    pub items: I,
    selected: usize,
    _phantom_v: PhantomData<T>,
    _phantom_f: PhantomData<F>,
}

impl<I, F: FnOnce() -> Action<Keys>>
    View<Keys> for Menu<I, Action<Keys>, F>
    where
        for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<Action<Keys>, F>>
{
    fn render(&mut self, context: Context<Keys>) -> Option<Action<Keys>> {
        let y_offset = 150;
        let y_gutter = 70;

        let mut len = 0;

        context.renderer.set_draw_color(Color::RGB(0, 0, 0));
        context.renderer.clear();


        for (i, item) in self.items.into_iter().enumerate() {
            len += 1;

            let sprite = if i == self.selected {
                if context.events.down.space {
                    return item.on_select.take().map(|a| a.consume());
                }

                &item.hover_sprite
            } else {
                &item.idle_sprite
            };

            let (w, h) = context.renderer.output_size().unwrap();

            context.renderer.copy_sprite(
                sprite,
                Dest {
                    x: ((w - sprite.mask.width) / 2) as _,
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
                (self.selected + len) - 1
            } else {
                self.selected
            }
        ) % len;

        None
    }
}

impl<I, T, F: FnOnce() -> T>
    Menu<I, T, F>
    where for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<T, F>>
{
    pub fn new(items: I) -> Self {
        Menu {
            items: items,
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
    ).coelesce().unwrap()
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

pub fn main_menu(
    renderer: &mut Renderer,
    cache: &mut FontCache,
    view: Box<View<Keys>>
) -> Menu<
    [MenuItem<Action<Keys>>; 2],
    Action<Keys>
> {
    let path = "assets/belligerent.ttf";
    let cache = cache.with_loaded(path, 38)
        .and_then(|c| c.with_loaded(path, 32))
        .unwrap();

    let font0 = cache.get(path, 32).unwrap();
    let font1 = cache.get(path, 38).unwrap();
    let fonts = (font0, font1);

    Menu::new(
        [
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
        ]
    )
}
