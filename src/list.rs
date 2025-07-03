#![allow(unused)]

struct Node<T> {
    elem: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(elem: T, next: Option<Box<Self>>) -> Self {
        Node { elem, next }
    }
}

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    tail: *mut Node<T>,
    size: u32,
}

impl<T: std::cmp::PartialEq> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None,
            tail: std::ptr::null_mut(),
            size: 0,
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn add(&mut self, elem: T) -> () {
        let mut node = Box::new(Node::new(elem, None));

        let new_tail: *mut Node<T> = &mut *node;

        if !self.tail.is_null() {
            unsafe {
                (*self.tail).next = Some(node);
            }
        } else {
            self.head = Some(node);
        }

        self.tail = new_tail;

        self.size += 1;
    }
    
    /// removes the first node from the list and returns its value.
    pub fn pop(&mut self) -> Option<T> {
        /// take() replaces the actual head by None an returns it's original value
        self.head.take().map(|h| {
            self.head = h.next;

            if self.head.is_none() {
                self.tail = std::ptr::null_mut();
            }

            self.size -= 1;

            h.elem
        })
    }

    /// removes the first node with value `elem`
    /// unlike [`pop`], you can choose which element to remove.
    pub fn remove(&mut self, elem: T) -> Option<T> {
        let mut node_it = &mut self.head;

        while !node_it.is_none() {
            /// Avoid borrow checker problems with this approach.
            let to_remove = node_it.as_ref().unwrap().elem == elem;

            if to_remove {
                let mut removed = node_it.take().unwrap();
                *node_it = removed.next.take();
                
                let empty = self.head.is_none();
                if empty {
                    self.tail = std::ptr::null_mut();
                }

                self.size -= 1;
                return Some(removed.elem);
            }
            
            let node = node_it.as_mut().unwrap();
            node_it = &mut node.next;
            
        }

        return None;
    }
}


#[cfg(test)]
mod tests {
    use super::LinkedList;

    #[test]
    fn basics() {
        let mut list = LinkedList::new();

        // Check empty list behaves correctly
        assert_eq!(list.size(), 0);
        assert_eq!(list.pop(), None);
        
        // Add elements to list
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.size(), 3);

        // Check pop
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.size(), 1);

        // Check remove
        list.add(4);
        list.add(5);
        list.add(6);
        list.add(7);
        assert_eq!(list.size(), 5);
        assert_eq!(list.remove(5), Some(5));
        assert_eq!(list.size(), 4);
        assert_eq!(list.remove(5), None);
    }
}