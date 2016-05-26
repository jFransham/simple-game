use ::events::KeySet;
use ::graphics::font_cache::FontCache;
use ::graphics::sprites::Renderable;
use ::gameobjects::Dest;

use sdl2::render::Renderer;

pub enum Action<'a, T: KeySet, R: Renderable<Renderer<'a>>> {
    Quit,
    ChangeView(Box<ViewBuilder<T, R>>),
    Render(Box<Iterator<Item=(R, Dest)> + 'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvents<T: KeySet> {
    pub down: T,
    pub pressed: T,
    pub released: T,
}

impl<T: KeySet> KeyEvents<T> {
    pub fn new(last: T, now: T) -> Self {
        let pressed = now.pressed_since(&last);
        let released = now.released_since(&last);

        KeyEvents {
            down: now,
            pressed: pressed,
            released: released,
        }
    }
}

pub struct Context<'a, 'b: 'a, T: KeySet> {
    pub events: KeyEvents<T>,
    pub renderer: &'a mut Renderer<'b>,
    pub font_cache: &'a mut FontCache<'b>,
}

pub trait View<T: KeySet, R: for<'a> Renderable<Renderer<'a>>> {
    fn update(
        &mut self,
        context: &mut Context<T>,
        elapsed: u32
    ) -> Action<T, R>;
}

#[allow(boxed_local)]
pub trait ViewBuilder<T: KeySet, R: for<'a> Renderable<Renderer<'a>>> {
    fn build_view(self: Box<Self>, context: &mut Context<T>) -> Box<View<T, R>>;
}

#[allow(boxed_local)]
impl<
    T: KeySet,
    R: for<'a> Renderable<Renderer<'a>>,
    F: FnOnce(&mut Context<T>) -> Box<View<T, R>>
> ViewBuilder<T, R> for F {
    fn build_view(
        self: Box<Self>,
        context: &mut Context<T>
    ) -> Box<View<T, R>> {
        self(context)
    }
}
