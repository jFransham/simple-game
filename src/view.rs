use ::events::KeySet;
use ::graphics::font_cache::FontCache;
use sdl2::render::Renderer;

pub enum Action<T: KeySet> {
    Quit,
    ChangeView(Box<ViewBuilder<T>>),
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

pub trait View<T: KeySet> {
    fn render(&mut self, context: &mut Context<T>, elapsed: u32) -> Option<Action<T>>;
}

#[allow(boxed_local)]
pub trait ViewBuilder<T: KeySet> {
    fn build_view(self: Box<Self>, context: &mut Context<T>) -> Box<View<T>>;
}

#[allow(boxed_local)]
impl<T: KeySet, F: FnOnce(&mut Context<T>) -> Box<View<T>>>
    ViewBuilder<T> for F
{
    fn build_view(self: Box<Self>, context: &mut Context<T>) -> Box<View<T>> {
        self(context)
    }
}
