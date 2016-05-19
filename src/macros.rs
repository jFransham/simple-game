macro_rules! key_set {
    (
        $set_name:ident {
            keyboard: {
                $( $key_name:ident : $( $key_code:pat )|*, )+
            }
            $(
                , else: {
                    $( $name:ident : $pat:pat ,)+
                }
            )*
        }
    ) => {
        #[derive(Clone, Debug, Default)]
        pub struct $set_name {
            $(
                pub $key_name: bool,
            )+
            $(
                $( pub $name: bool, )*
            )*
        }

        impl KeySet for $set_name {
            fn from_keycode_iterator<T: Iterator<Item=Event>>(
                &self,
                iter: T
            ) -> Self {
                use sdl2::event::Event::*;
                use sdl2::keyboard::Keycode::*;

                let mut out = self.clone();

                for e in iter {
                    match e {
                        KeyDown {
                            keycode: Some(kc),
                            ..
                        } => match kc {
                            $(
                                $( $key_code )|* => out.$key_name = true,
                            )*
                            _ => { }
                        },
                        KeyUp {
                            keycode: Some(kc),
                            ..
                        } => match kc {
                            $(
                                $( $key_code )|* => out.$key_name = false,
                            )*
                            _ => { }
                        },
                        $(
                            $(
                                $pat => out.$name = true,
                            )*
                        )*
                        _ => { }
                    }
                }

                out
            }

            fn pressed_since(&self, last: &Self) -> Self {
                $set_name {
                    $(
                        $key_name: !last.$key_name && self.$key_name,
                    )+
                    $(
                        $( $name: !last.$name && self.$name, )*
                    )*
                }
            }

            fn released_since(&self, other: &Self) -> Self {
                other.pressed_since(self)
            }
        }
    };
}
