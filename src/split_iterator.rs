use std::marker::PhantomData;

pub struct SplitIterMut<'a, T: 'a> {
    ptr: *mut T,
    end: *mut T,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: 'a> Iterator for SplitIterMut<'a, T> {
    type Item = (&'a mut T, &'a mut [T]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.end {
            None
        } else {
            use std::mem::{transmute, size_of};
            use std::slice::from_raw_parts_mut;

            let out: &mut T = unsafe { transmute(self.ptr) };
            self.ptr = unsafe { self.ptr.offset(1) };
            let rest: &mut [T] = unsafe {
                from_raw_parts_mut(
                    self.ptr,
                    (self.end as usize - self.ptr as usize) / size_of::<T>()
                    )
            };
            Some((out, rest))
        }
    }
}

pub trait IntoSplitIterMut {
    type Item;

    fn split_iter_mut(&mut self) -> SplitIterMut<Self::Item>;
}

impl<T> IntoSplitIterMut for [T] {
    type Item = T;

    fn split_iter_mut(&mut self) -> SplitIterMut<T> {
        let ptr = self.as_mut_ptr();

        SplitIterMut {
            ptr: ptr,
            end: unsafe { ptr.offset(self.len() as isize) },
            _marker: PhantomData,
        }
    }
}
