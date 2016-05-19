use std::mem;
use std::boxed::FnBox;

pub enum Lazy<T, F: FnOnce() -> T = Box<FnBox() -> T>> {
    Value(T),
    Function(F),
    Poisoned,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    pub fn consume(self) -> T {
        use self::Lazy::*;

        match self {
            Value(v) => v,
            Function(f) => f(),
            Poisoned => panic!("Lazy<T, F> is poisoned."),
        }
    }

    pub fn consume_in_place(&mut self) {
        use self::Lazy::*;

        let v = match mem::replace(self, Poisoned) {
            Function(f) => f(),
            Value(v) => v,
            Poisoned => panic!("Lazy<T, F> is poisoned."),
        };

        *self = Value(v);
    }
}
