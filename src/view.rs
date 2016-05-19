use ::events::{KeySet, GameTime};
use sdl2::render::Renderer;

pub enum Action<T: KeySet> {
    Quit,
    ChangeView(Box<View<T>>),
}

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
    pub time: GameTime,
    pub events: KeyEvents<T>,
    pub renderer: &'a mut Renderer<'b>,
}

pub trait View<T: KeySet> {
    fn render(&mut self, context: Context<T>) -> Option<Action<T>>;
}
