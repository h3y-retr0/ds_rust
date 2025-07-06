use std::{
    alloc, marker, mem,
    ops::{Deref, DerefMut},
    ptr,
    ptr::NonNull,
};

/// Buffer of fixed capacity that stores the values.
struct Buffer<T> {
    ptr: NonNull<T>,
    cap: usize,
    _marker: marker::PhantomData<T>,
}

// unsafe impl<T: Send> Send for Vec<T> {}
// unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Buffer<T> {
    /// Create a new RawVec with zero capacity.
    pub fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            0
        };

        Self {
            ptr: NonNull::dangling(),
            cap,
            _marker: marker::PhantomData,
        }
    }

    /// Allocates a new buffer if the capacity is zero, otherwise it doubles
    /// the size of the buffer and reallocates it.
    fn grow(&mut self) {
        // We shouldn't get to this point if `T` is zero sized.
        assert!(mem::size_of::<T>() != 0, "Capacity overflow");

        let (new_cap, new_layout, new_ptr) = if self.cap == 0 {
            let new_layout = alloc::Layout::array::<T>(1).unwrap();
            let new_ptr = unsafe { alloc::alloc(new_layout) };

            (1, new_layout, new_ptr)
        } else {
            let new_cap = self.cap * 2;
            let new_layout = alloc::Layout::array::<T>(new_cap).unwrap();

            assert!(
                new_layout.size() <= isize::MAX as usize,
                "Allocation too large"
            );

            let new_ptr = unsafe {
                alloc::realloc(
                    self.ptr.as_ptr() as *mut u8,
                    alloc::Layout::array::<T>(self.cap).unwrap(),
                    new_layout.size(),
                )
            };

            (new_cap, new_layout, new_ptr)
        };

        self.ptr = match ptr::NonNull::new(new_ptr as *mut T) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(new_layout),
        };

        self.cap = new_cap;
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        if self.cap != 0 && mem::size_of::<T>() != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    alloc::Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}
/// List data structure stored as an array.
pub struct Vector<T> {
    buf: Buffer<T>,
    len: usize,
}

impl<T> Vector<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    /// Returns the capacity of the buffer.
    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Creates and returns a new `Vec` with zero length.
    pub fn new() -> Self {
        Self {
            buf: Buffer::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::write(self.ptr().add(self.len), value);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "Index out of bounds");

        if self.cap() == self.len {
            self.buf.grow();
        }

        unsafe {
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );

            ptr::write(self.ptr().add(index), value);

            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");

        unsafe {
            let value = ptr::read(self.ptr().add(index));

            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );

            value
        }
    }

    pub fn drain(&mut self) -> Drain<T> {
        let iter = RawIter::new(&self);

        self.len = 0;

        Drain {
            iter,
            vec: marker::PhantomData,
        }
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let iter = RawIter::new(&self);

            let buf = ptr::read(&self.buf);

            mem::forget(self);

            IntoIter { iter, _buf: buf }
        }
    }
}

struct RawIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawIter<T> {
    fn new(slice: &[T]) -> Self {
        RawIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                (slice.as_ptr() as usize + slice.len()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                unsafe { slice.as_ptr().add(slice.len()) }
            },
        }
    }
}

impl<T> Iterator for RawIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        unsafe {
            if mem::size_of::<T>() == 0 {
                self.start = (self.start as usize + 1) as *const _;
                Some(ptr::read(ptr::NonNull::<T>::dangling().as_ptr()))
            } else {
                let old_ptr = self.start;
                self.start = self.start.offset(1);
                Some(ptr::read(old_ptr))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let type_size = mem::size_of::<T>();

        let mut len = self.end as usize - self.start as usize;

        if type_size != 0 {
            len /= type_size;
        }

        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        unsafe {
            if mem::size_of::<T>() == 0 {
                self.end = (self.end as usize - 1) as *const _;
                Some(ptr::read(ptr::NonNull::<T>::dangling().as_ptr()))
            } else {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}

pub struct IntoIter<T> {
    _buf: Buffer<T>,
    iter: RawIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

pub struct Drain<'a, T: 'a> {
    vec: marker::PhantomData<&'a mut Vector<T>>,
    iter: RawIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

#[cfg(test)]
mod tests {
    use super::Vector;

    #[test]
    fn test_basics() {
        let mut v = Vector::<i32>::new();

        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);

        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.len(), 2);
    }
}