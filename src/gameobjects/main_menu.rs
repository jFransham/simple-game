use ::time::TimeExtensions;
use ::coalesce::Coalesce;
use ::events::Keys;
use ::view::{Context, View, ViewBuilder, Action};
use ::graphics::sprites::{LoadSprite, CopyRenderable, Sprite, VisibleComponent};
use ::graphics::font_cache::FontCache;
use ::gameobjects::background::ParallaxSet;
use ::gameobjects::player::*;
use ::gameobjects::Dest;

use std::marker::PhantomData;
use sdl2_ttf::Font;
use sdl2::render::{Texture, Renderer};
use sdl2::pixels::Color;

pub type Background = [([f64; 2], [f64; 2], Sprite<Texture>); 3];

pub struct Menu<
    Time: TimeExtensions,
    I,
    B,
    T
> where
    //for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<T>>,
    for<'a> &'a B: IntoIterator<Item=&'a ([f64; 2], [f64; 2], Sprite<Texture>)>
{
    pub items: I,
    background: ParallaxSet<Time, Texture, B>,
    total_time: u32,
    count: usize,
    selected: usize,
    _phantom_v: PhantomData<T>,
}

impl<'any, I, B> View<Keys, VisibleComponent<Texture>>
    for Menu<u32, I, B, Action<'any, Keys, VisibleComponent<Texture>>>
    where
        for<'a> &'a mut I: IntoIterator<
            Item=&'a mut MenuItem<Action<'any, Keys, VisibleComponent<Texture>>>
        >,
        for<'a> &'a B: IntoIterator<Item=&'a ([f64; 2], [f64; 2], Sprite<Texture>)>
{
    fn update(
        &mut self,
        context: &mut Context<Keys>,
        elapsed: u32
    ) -> Action<Keys, VisibleComponent<Texture>> {
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

        if context.events.down.fire {
            if let Some(slct) = self.items.into_iter()
                .enumerate()
                .skip(self.selected)
                .next()
                .and_then(|(_, item)| item.on_select.take())
            {
                return slct;
            }
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

        let selected = self.selected;

        Action::Render(
            box self.items.into_iter().enumerate().map(
                move |(i, item)| {
                    let sprite = if i == selected {
                        &item.hover_sprite
                    } else {
                        &item.idle_sprite
                    };

                    (
                        sprite.clone().into(),
                        Dest {
                            x: ((screen_w - sprite.mask.width) / 2) as _,
                            y:
                                (y_offset + y_gutter * i) as i32 -
                                (sprite.mask.height / 2) as i32,
                            width: sprite.mask.width,
                            height: sprite.mask.height,
                        }
                    )
                }
            )
        )
    }
}

impl<Time: TimeExtensions + Copy + Default, I, T>
    Menu<Time, I, [([f64; 2], [f64; 2], Sprite<Texture>); 3], T>
    where
        for<'a> &'a mut I: IntoIterator<Item=&'a mut MenuItem<T>>,
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
        }
    }
}

pub struct MenuItem<T> {
    idle_sprite: Sprite<Texture>,
    hover_sprite: Sprite<Texture>,
    on_select: Option<T>,
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

impl<T> MenuItem<T> {
    pub fn new(
        idle: Sprite<Texture>,
        hover: Sprite<Texture>,
        v: T
    ) -> Self {
        MenuItem {
            idle_sprite: idle,
            hover_sprite: hover,
            on_select: Some(v),
        }
    }
}

pub struct MainMenuBuilder;

#[allow(boxed_local)]
impl ViewBuilder<Keys, VisibleComponent<Texture>> for MainMenuBuilder {
    fn build_view(self: Box<Self>, context: &mut Context<Keys>)
        -> Box<View<Keys, VisibleComponent<Texture>>>
    {
        Box::new(
            main_menu(context.renderer, context.font_cache, box ShipViewBuilder)
        )
    }
}

pub struct PauseMenuBuilder<A, B>(pub Box<View<A, B>>);

#[allow(boxed_local)]
impl ViewBuilder<Keys, VisibleComponent<Texture>>
    for PauseMenuBuilder<Keys, VisibleComponent<Texture>>
{
    fn build_view(self: Box<Self>, context: &mut Context<Keys>)
        -> Box<View<Keys, VisibleComponent<Texture>>>
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

pub fn main_menu<'a>(
    renderer: &mut Renderer,
    cache: &mut FontCache,
    view: Box<ViewBuilder<Keys, VisibleComponent<Texture>>>
) -> Menu<
    u32,
    [MenuItem<Action<'a, Keys, VisibleComponent<Texture>>>; 2],
    Background,
    Action<'a, Keys, VisibleComponent<Texture>>
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
            MenuItem::new(
                idle,
                hover,
                Action::ChangeView(view)
            )
        },
        {
            let (idle, hover) = get_sprites(renderer, "Quit", fonts);
            MenuItem::new(idle, hover, Action::Quit)
        },
    ];

    Menu::new(renderer, items)
}
