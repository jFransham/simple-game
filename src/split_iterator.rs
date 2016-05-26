use std::marker::PhantomData;

pub struct SplitIterMut<
    'a,
    T: 'a,
    O,
    F: for<'b> FnMut(&'b mut T, &'b mut [T]) -> O
> {
    inner: &'a mut [T],
    counter: usize,
    f: F,
    _marker_out: PhantomData<O>,
}

impl<
    'a,
    T: 'a,
    O,
    F: for<'b> FnMut(&'b mut T, &'b mut [T]) -> O
> Iterator for SplitIterMut<'a, T, O, F> {
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == self.inner.len() { return None; }

        let counter = self.counter;
        let f = &mut self.f;

        self.counter += 1;

        self.inner[counter..].split_first_mut().map(
            move |(first, rest)| {
                f(first, rest)
            }
        )
    }
}

pub trait IntoSplitIterMut {
    type Item;

    fn split_iter_mut<
        O,
        F: for<'a> FnMut(&'a mut Self::Item, &'a mut [Self::Item]) -> O
    >(&mut self, func: F) -> SplitIterMut<Self::Item, O, F>;
}

impl<T> IntoSplitIterMut for [T] {
    type Item = T;

    fn split_iter_mut<
        O,
        F: for<'a> FnMut(&'a mut Self::Item, &'a mut [Self::Item]) -> O
    >(&mut self, func: F) -> SplitIterMut<T, O, F> {
        SplitIterMut {
            inner: self,
            counter: 0,
            f: func,
            _marker_out: PhantomData,
        }
    }
}
