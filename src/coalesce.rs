pub trait Coalesce {
    type Output;

    fn coalesce(self) -> Self::Output;
}

macro_rules! impl_coalesce {
    ($head:ident,) => {
        impl<$head> Coalesce for (Option<$head>,) {
            type Output = Option<($head,)>;

            #[allow(non_snake_case)]
            fn coalesce(self) -> Self::Output {
                self.0.map(|h| (h,))
            }
        }
    };
    ($head:ident, $( $rest:ident ,)+) => {
        impl<$head $( , $rest )+> Coalesce for (
            Option<$head>,
            $( Option<$rest> ,)+
        ) {
            type Output = Option<($head, $( $rest ,)+)>;

            #[allow(non_snake_case)]
            fn coalesce(self) -> Self::Output {
                let ($head, $( $rest ,)+) = self;

                $head.and_then(|$head|
                    ($( $rest ,)+).coalesce().map(
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

        impl_coalesce!($( $rest ,)+);
    };
}

impl_coalesce!(A, B, C, D, E, F, G, H, I, J, K,);
