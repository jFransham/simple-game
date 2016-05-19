pub trait Coelesce {
    type Output;

    fn coelesce(self) -> Self::Output;
}

macro_rules! impl_coelesce {
    ($head:ident,) => {
        impl<$head> Coelesce for (Option<$head>,) {
            type Output = Option<($head,)>;

            #[allow(non_snake_case)]
            fn coelesce(self) -> Self::Output {
                self.0.map(|h| (h,))
            }
        }
    };
    ($head:ident, $( $rest:ident ,)+) => {
        impl<$head $( , $rest )+> Coelesce for (
            Option<$head>,
            $( Option<$rest> ,)+
        ) {
            type Output = Option<($head, $( $rest ,)+)>;

            #[allow(non_snake_case)]
            fn coelesce(self) -> Self::Output {
                let ($head, $( $rest ,)+) = self;

                $head.and_then(|$head|
                    ($( $rest ,)+).coelesce().map(
                        |($( $rest ,)+)|
                            (
                                $head,
                                $(
                                    $rest,
                                )+
                            )
                    )
                )
            }
        }

        impl_coelesce!($( $rest ,)+);
    };
}

impl_coelesce!(A, B, C, D, E, F, G, H, I, J, K,);
