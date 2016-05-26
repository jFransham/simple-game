pub trait FixedSizeIntoMap<O> {
    type Item;
    type Out;

    fn map<F: FnMut(Self::Item) -> O>(self, f: F) -> Self::Out;
}

pub trait FixedSizeMap<O> {
    type Item;
    type Out;

    fn map<F: FnMut(Self::Item) -> O>(&self, f: F) -> Self::Out;
}

macro_rules! impl_fixed_size_map {
    ($n:expr) => {
        /// WARNING: THIS IS NOT PANIC-SAFE. If f panics then God help you and
        /// your laundry. Don't use this if you do not control f and T is Drop.
        impl<T, O> FixedSizeIntoMap<O> for [T; $n] {
            type Item = T;
            type Out = [O; $n];

            fn map<F: FnMut(Self::Item) -> O>(mut self, mut f: F) -> Self::Out {
                use std::mem::{forget, uninitialized, replace};
                use std::ptr::write;

                let mut out: Self::Out = unsafe { uninitialized() };

                for i in 0..$n {
                    // So we don't drop uninitialized memory
                    unsafe {
                        write(
                            (&mut out[i]) as _,
                            f(replace(&mut self[i], uninitialized()))
                        )
                    };
                }

                forget(self);

                out
            }
        }
    }
}

impl_fixed_size_map!(1);
impl_fixed_size_map!(2);
impl_fixed_size_map!(3);
impl_fixed_size_map!(4);
impl_fixed_size_map!(5);
impl_fixed_size_map!(6);
impl_fixed_size_map!(7);
impl_fixed_size_map!(8);
impl_fixed_size_map!(9);
impl_fixed_size_map!(10);
impl_fixed_size_map!(11);
impl_fixed_size_map!(12);
impl_fixed_size_map!(13);
impl_fixed_size_map!(14);
impl_fixed_size_map!(15);
impl_fixed_size_map!(16);
impl_fixed_size_map!(17);
impl_fixed_size_map!(18);
impl_fixed_size_map!(19);
impl_fixed_size_map!(20);
impl_fixed_size_map!(21);
impl_fixed_size_map!(22);
impl_fixed_size_map!(23);
impl_fixed_size_map!(24);
impl_fixed_size_map!(25);
impl_fixed_size_map!(26);
impl_fixed_size_map!(27);
impl_fixed_size_map!(28);
impl_fixed_size_map!(29);
impl_fixed_size_map!(30);
impl_fixed_size_map!(31);
impl_fixed_size_map!(32);
impl_fixed_size_map!(33);
impl_fixed_size_map!(34);
impl_fixed_size_map!(35);
impl_fixed_size_map!(36);
impl_fixed_size_map!(37);
impl_fixed_size_map!(38);
impl_fixed_size_map!(39);
impl_fixed_size_map!(40);
impl_fixed_size_map!(41);
impl_fixed_size_map!(42);
impl_fixed_size_map!(43);
impl_fixed_size_map!(44);
impl_fixed_size_map!(45);
impl_fixed_size_map!(46);
impl_fixed_size_map!(47);
impl_fixed_size_map!(48);
impl_fixed_size_map!(49);
impl_fixed_size_map!(50);
