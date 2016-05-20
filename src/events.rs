use std::marker::PhantomData;
use std::default::Default;
use sdl2::EventPump;
pub use sdl2::event::Event;

pub trait KeySet: Default {
    fn from_keycode_iterator<T: Iterator<Item=Event>>(&self, _: T) -> Self;
    fn pressed_since(&self, _: &Self) -> Self;
    fn released_since(&self, other: &Self) -> Self;
}

key_set! {
    Keys {
        keyboard: {
            escape: Escape,
            up: Up,
            down: Down,
            left: Left,
            right: Right,
            space: Space,
        },
        else: {
            quit: Quit { .. },
        }
    }
}

pub struct EventStream<T: KeySet> {
    pump: EventPump,
    _out: PhantomData<T>,
}

#[derive(Debug, Copy, Clone)]
pub struct GameTime {
    pub elapsed: u32,
    pub total: u32,
}

impl<T: KeySet> EventStream<T> {
    pub fn new(pump: EventPump) -> EventStream<T> {
        EventStream {
            pump: pump,
            _out: PhantomData,
        }
   }

    pub fn pump(&mut self, last: &T) -> T {
        last.from_keycode_iterator(self.pump.poll_iter())
    }
}
