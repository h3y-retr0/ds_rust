use std::{fmt::Debug, hash::Hash, marker::PhantomData, ptr::NonNull};

struct Node<T> {
    next: Link<T>,
    prev: Link<T>,
    elem: T,
}

type Link<T> = Option<NonNull<Node<T>>>;

pub struct DequeueList<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    marker: PhantomData<T>
}

pub struct Iter<'a, T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    marker: PhantomData<&'a mut T>
}

pub struct IterMut<'a, T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    marker: PhantomData<&'a T>,
}

pub struct IntoIter<T>(DequeueList<T>);

pub struct CursorMut<'a, T> {
    current: Link<T>,
    list: &'a mut DequeueList<T>,
    index: Option<usize>,
}

impl<T> Node<T> {
    fn new(next: Link<T>, prev: Link<T>, elem: T) -> NonNull<Node<T>> {
        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node { next, prev, elem })))
        }
    }
}

impl<T> DequeueList<T> {
    pub fn new() -> Self {
        DequeueList {
            head: None,
            tail: None,
            len: 0,
            marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) -> () {
        while self.pop_front().is_some() {}
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            let new_node = Node::new(None, None, elem);

            if let Some(old_head) = self.head {
                (*old_head.as_ptr()).prev = Some(new_node);
                (*new_node.as_ptr()).next = Some(old_head);
            } else {
                self.tail = Some(new_node);
            }

            self.head = Some(new_node);
            self.len += 1;
        }
    }

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new_node = Node::new(None, None, elem);

            if let Some(old_tail) = self.tail {
                (*old_tail.as_ptr()).next = Some(new_node);
                (*new_node.as_ptr()).prev = Some(old_tail); 
            } else {
                self.head = Some(new_node);
            }

            self.tail = Some(new_node);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.map(|node| unsafe {
            let current_head = Box::from_raw(node.as_ptr());
            let elem = current_head.elem;

            self.head = current_head.next;

            if let Some(new_head) = self.head {
                (*new_head.as_ptr()).prev = None
            } else {
                self.tail = None
            }

            self.len -= 1;

            elem
        }) 
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.map(|node| unsafe {
            let current_tail = Box::from_raw(node.as_ptr());
            let elem = current_tail.elem;

            self.tail = current_tail.prev;

            if let Some(new_tail) = self.tail {
                (*new_tail.as_ptr()).next = None;
            } else {
                self.head = None;
            }

            self.len -= 1;

            elem
        })
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { Some(&(*self.head?.as_ptr()).elem) }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { Some(&mut (*self.head?.as_ptr()).elem) }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe { Some(&(*self.tail?.as_ptr()).elem) }
    }

    pub fn back_mut(&self) -> Option<&mut T> {
        unsafe { Some(&mut (*self.tail?.as_ptr()).elem) }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        CursorMut { 
            current: None, 
            list: self, 
            index: None 
        }
    }
}


impl<T> Drop for DequeueList<T> {
    
    /// See [`DequeueList::clear`] for a different implementation of this loop.
    fn drop(&mut self) {
        // Pop elements until we have to stop.
        while let Some(_) = self.pop_front() { }
    }
}

impl<'a, T> IntoIterator for &'a DequeueList<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.head.map(|node| unsafe {
            self.len -= 1;
            self.head = (*node.as_ptr()).next;
            &(*node.as_ptr()).elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl <'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.tail.map(|node| unsafe {
            self.len -= 1;
            self.tail = (*node.as_ptr()).prev;
            &(*node.as_ptr()).elem
        })
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> IntoIterator for &'a mut DequeueList<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.head.map(|node| unsafe {
            self.len -= 1;
            self.head = (*node.as_ptr()).next;
            &mut (*node.as_ptr()).elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.tail.map(|node| unsafe {
            self.len -= 1;
            self.tail = (*node.as_ptr()).prev;
            &mut (*node.as_ptr()).elem
        })
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> IntoIterator for DequeueList<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len, Some(self.0.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.0.len
    }
}

impl<T> Default for DequeueList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for DequeueList<T> {
    fn clone(&self) -> Self {
        let mut new_dequeue = Self::new();

        for value in self {
            new_dequeue.push_back(value.clone())
        }

        new_dequeue
    }
}

impl<T> Extend<T> for DequeueList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T> FromIterator<T> for DequeueList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut dequeue = Self::new();
        dequeue.extend(iter);

        dequeue
    }
}

impl<T: Debug> Debug for DequeueList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: PartialEq> PartialEq for DequeueList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
    }
}

impl<T: Eq> Eq for DequeueList<T> { }

impl<T: PartialOrd> PartialOrd for DequeueList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}
 
impl<T: Ord> Ord for DequeueList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash> Hash for DequeueList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self {
            item.hash(state);
        }
    }
}

// DOCS COPIED FROM BOOK

// A Cursor is like an iterator, except that it can freely seek back-and-forth, 
// and can safely mutate the list during iteration. This is because the lifetime 
// of its yielded references are tied to its own lifetime, instead of just the underlying list. 
// This means cursors cannot yield multiple elements at once.

// Cursors always rest between two elements in the list, and index in a logically circular way. 
// To accomadate this, there is a "ghost" non-element that yields None between the head and tail of the List.

// When created, cursors start between the ghost and the front of the list. 
// That is, next will yield the front of the list, and prev will yield None. 
// Calling prev again will yield the tail.


impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn move_next(&mut self) {
        if let Some(current) = self.current {
            unsafe {
                self.current = (*current.as_ptr()).next;
                if self.current.is_some() {
                    *self.index.as_mut().unwrap() += 1;
                } else {
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            self.current = self.list.head;
            self.index = Some(0);
        } else {
            // Ghost
        }
    }

    pub fn move_prev(&mut self) {
        if let Some(current) = self.current {
            unsafe {
                self.current = (*current.as_ptr()).prev;
                if self.current.is_some() {
                    *self.index.as_mut().unwrap() -= 1;
                } else {
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            self.current = self.list.tail;
            self.index = Some(self.list.len - 1);
        } else {
            // Ghost
        }
    }

    pub fn current(&mut self) -> Option<&mut T> {
        unsafe { self.current.map(|node| &mut (*node.as_ptr()).elem) }
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        unsafe {
            let next = if let Some(current) = self.current {
                (*current.as_ptr()).next
            } else {
                self.list.head
            };

            next.map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        unsafe {
            let prev = if let Some(current) = self.current {
                (*current.as_ptr()).prev
            } else {
                self.list.tail
            };

            prev.map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn split_before(&mut self) -> DequeueList<T> {
        if self.current.is_none() {
            return std::mem::replace(self.list, DequeueList::new());
        }

        unsafe {
            let current = self.current.unwrap();

            let old_len = self.list.len;
            let old_idx = self.index.unwrap();
            let prev = (*current.as_ptr()).prev;

            let new_len = old_len - old_idx;
            let new_head = self.current;
            let new_tail = self.list.tail;
            let new_idx = Some(0);

            let output_len = old_len - new_len;
            let output_head = self.list.head;
            let output_tail = prev;

            if let Some(prev) = prev {
                (*current.as_ptr()).prev = None;
                (*prev.as_ptr()).next = None;
            }

            self.list.len = new_len;
            self.list.head = new_head;
            self.list.tail = new_tail;
            self.index = new_idx;

            DequeueList {
                head: output_head,
                tail: output_tail,
                len: output_len,
                marker: PhantomData,
            }
        }
    }

    pub fn split_after(&mut self) -> DequeueList<T> {
        if self.current.is_none() {
            return std::mem::replace(self.list, DequeueList::new());
        }

        unsafe {
            let current = self.current.unwrap();

            let old_len = self.list.len;
            let old_idx = self.index.unwrap();
            let next = (*current.as_ptr()).next;

            let new_len = old_idx + 1;
            let new_head = self.list.head;
            let new_tail = self.current;
            let new_idx = Some(old_idx);

            let output_len = old_len - new_len;
            let output_head = next;
            let output_tail = self.list.tail;

            if let Some(next) = next {
                (*current.as_ptr()).next = None;
                (*next.as_ptr()).prev = None;
            }

            self.list.len = new_len;
            self.list.tail = new_tail;
            self.list.head = new_head;
            self.index = new_idx;

            DequeueList {
                tail: output_tail,
                head: output_head,
                len: output_len,
                marker: PhantomData,
            }
        }
    }

    pub fn splice_before(&mut self, mut input: DequeueList<T>) {
        if input.is_empty() {
            return;
        }

        if self.list.is_empty() {
            *self.list = input;
            return;
        }

        let input_head = input.head.take().unwrap();
        let input_tail = input.tail.take().unwrap();

        unsafe {
            if let Some(current) = self.current {
                if let Some(prev) = (*current.as_ptr()).prev {
                    (*prev.as_ptr()).next = Some(input_head);
                    (*input_head.as_ptr()).prev = Some(prev);
                    (*current.as_ptr()).prev = Some(input_tail);
                    (*input_tail.as_ptr()).next = Some(current);
                } else{
                    (*current.as_ptr()).prev = Some(input_tail);
                    (*input_tail.as_ptr()).next = Some(current);
                    self.list.head = Some(input_head);
                }
            } else {
                (*self.list.tail.unwrap().as_ptr()).next = Some(input_head);
                (*input_head.as_ptr()).prev = self.list.tail;
                self.list.tail = Some(input_tail);
            }

            self.list.len += input.len;
            input.len = 0;
        }
    }

    pub fn splice_after(&mut self, mut input: DequeueList<T>) {
        if input.is_empty() {
            return;
        }

        if self.list.is_empty() {
            *self.list = input;
            return;
        }

        let input_head = input.head.take().unwrap();
        let input_tail = input.tail.take().unwrap();

        unsafe {
            if let Some(current) = self.current {
                if let Some(next) = (*current.as_ptr()).next {
                    (*next.as_ptr()).prev = Some(input_tail);
                    (*input_tail.as_ptr()).next = Some(next);
                    (*current.as_ptr()).next = Some(input_head);
                    (*input_head.as_ptr()).prev = Some(current);
                } else {
                    (*current.as_ptr()).next = Some(input_head);
                    (*input_head.as_ptr()).prev = Some(current);
                    self.list.tail = Some(input_tail);
                }
            } else {
                (*self.list.head.unwrap().as_ptr()).prev = Some(input_tail);
                (*input_tail.as_ptr()).next = self.list.head;
                self.list.head = Some(input_head);
            }

            self.list.len += input.len;
            input.len = 0;
        }
    }

    pub fn remove_current(&mut self) -> Option<T> {
        if self.list.is_empty() {
            return None;
        }

        if self.current.is_none() {
            return None;
        }

        unsafe {
            let mut current = Box::from_raw(self.current.unwrap().as_ptr());

            let value = current.elem;

            if let Some(next) = current.next {
                if let Some(prev) = current.prev {
                    (*prev.as_ptr()).next = Some(next);
                    (*next.as_ptr()).prev = Some(prev);
                } else {
                    self.list.head = Some(next);
                    (*next.as_ptr()).prev = None;
                }
                self.current = Some(next);
            } else {
                if let Some(prev) = current.prev {
                    self.list.tail = Some(prev);
                    (*prev.as_ptr()).next = None;
                } else {
                    self.list.tail = None;
                    self.list.head = None;
                }
                self.current = None;
            }

            current.next = None;
            current.prev = None;

            self.list.len -= 1;
            
            Some(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DequeueList;

    fn generate_test() -> DequeueList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(v: &[T]) -> DequeueList<T> {
        v.iter().map(|val| (*val).clone()).collect()
    }

    #[test]
    fn test_basic_front() {
        let mut list: DequeueList<i32> = DequeueList::new();

        // Test empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.len(), 0);

        // One item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_basic() {
        let mut m = DequeueList::new();
        assert_eq!(m.pop_front(), None);
        assert_eq!(m.pop_back(), None);
        assert_eq!(m.pop_front(), None);
        m.push_front(1);
        assert_eq!(m.pop_front(), Some(1));
        m.push_back(2);
        m.push_back(3);
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_front(), Some(2));
        assert_eq!(m.pop_front(), Some(3));
        assert_eq!(m.len(), 0);
        assert_eq!(m.pop_front(), None);
        m.push_back(1);
        m.push_back(3);
        m.push_back(5);
        m.push_back(7);
        assert_eq!(m.pop_front(), Some(1));

        let mut n = DequeueList::new();
        n.push_front(2);
        n.push_front(3);
        {
            assert_eq!(n.front().unwrap(), &3);
            let x = n.front_mut().unwrap();
            assert_eq!(*x, 3);
            *x = 0;
        }
        {
            assert_eq!(n.back().unwrap(), &2);
            let y = n.back_mut().unwrap();
            assert_eq!(*y, 2);
            *y = 1;
        }
        assert_eq!(n.pop_front(), Some(0));
        assert_eq!(n.pop_front(), Some(1));
    }

    #[test]
    fn test_iterator() {
        let m = generate_test();
        for (i, elt) in m.iter().enumerate() {
            assert_eq!(i as i32, *elt);
        }
        let mut n = DequeueList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_double_end() {
        let mut n = DequeueList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next_back().unwrap(), &4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next_back().unwrap(), &5);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_rev_iter() {
        let m = generate_test();
        for (i, elt) in m.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }
        let mut n = DequeueList::new();
        assert_eq!(n.iter().rev().next(), None);
        n.push_front(4);
        let mut it = n.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_mut_iter() {
        let mut m = generate_test();
        let mut len = m.len();
        for (i, elt) in m.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);
        let mut n = DequeueList::new();
        assert!(n.iter_mut().next().is_none());
        n.push_front(4);
        n.push_back(5);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_mut_double_end() {
        let mut n = DequeueList::new();
        assert!(n.iter_mut().next_back().is_none());
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn test_eq() {
        let mut n: DequeueList<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert!(n == m);
        n.push_front(1);
        assert!(n != m);
        m.push_back(1);
        assert!(n == m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert!(n != m);
    }

    #[test]
    fn test_ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn test_ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn test_debug() {
        let list: DequeueList<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: DequeueList<&str> = vec!["just", "one", "test", "more"]
            .iter()
            .copied()
            .collect();
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn test_hashmap() {
        // Check that HashMap works with this as a key

        let list1: DequeueList<i32> = (0..10).collect();
        let list2: DequeueList<i32> = (1..11).collect();
        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }

    #[test]
    fn test_cursor_move_peek() {
        let mut m: DequeueList<u32> = DequeueList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 1));
        assert_eq!(cursor.peek_next(), Some(&mut 2));
        assert_eq!(cursor.peek_prev(), None);
        assert_eq!(cursor.index(), Some(0));
        cursor.move_prev();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 2));
        assert_eq!(cursor.peek_next(), Some(&mut 3));
        assert_eq!(cursor.peek_prev(), Some(&mut 1));
        assert_eq!(cursor.index(), Some(1));

        let mut cursor = m.cursor_mut();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 6));
        assert_eq!(cursor.peek_next(), None);
        assert_eq!(cursor.peek_prev(), Some(&mut 5));
        assert_eq!(cursor.index(), Some(5));
        cursor.move_next();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 5));
        assert_eq!(cursor.peek_next(), Some(&mut 6));
        assert_eq!(cursor.peek_prev(), Some(&mut 4));
        assert_eq!(cursor.index(), Some(4));
    }

    #[test]
    fn test_cursor_mut_insert() {
        let mut m: DequeueList<u32> = DequeueList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.splice_before(Some(7).into_iter().collect());
        cursor.splice_after(Some(8).into_iter().collect());
        // check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[7, 1, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        cursor.splice_before(Some(9).into_iter().collect());
        cursor.splice_after(Some(10).into_iter().collect());
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[10, 7, 1, 8, 2, 3, 4, 5, 6, 9]
        );

        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(7));
        cursor.move_prev();
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), Some(9));
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(10));
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[1, 8, 2, 3, 4, 5, 6]
        );
        assert_eq!(m.len(), 7);

        let mut m: DequeueList<u32> = DequeueList::new();
        m.extend([1, 8, 2, 3, 4, 5, 6]);

        let mut cursor = m.cursor_mut();
        cursor.move_next();
        let mut p: DequeueList<u32> = DequeueList::new();
        p.extend([100, 101, 102, 103]);
        let mut q: DequeueList<u32> = DequeueList::new();
        q.extend([200, 201, 202, 203]);
        cursor.splice_after(p);
        cursor.splice_before(q);
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        let tmp = cursor.split_before();
        assert_eq!(m.into_iter().collect::<Vec<_>>(), &[]);
        m = tmp;
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        let tmp = cursor.split_after();
        assert_eq!(
            tmp.into_iter().collect::<Vec<_>>(),
            &[102, 103, 8, 2, 3, 4, 5, 6]
        );
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101]
        );
    }

    fn check_links<T: Eq + std::fmt::Debug>(list: &DequeueList<T>) {
        let from_front: Vec<_> = list.iter().collect();
        let from_back: Vec<_> = list.iter().rev().collect();
        let re_reved: Vec<_> = from_back.into_iter().rev().collect();

        assert_eq!(from_front, re_reved);
    }
}