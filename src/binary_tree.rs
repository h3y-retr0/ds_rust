use std::{ marker::PhantomData, ptr::NonNull, ptr };

/// BTree node.
struct Node<T> {
    left: Link<T>,
    right: Link<T>,
    elem: T,
}

/// Rusty pointers to nodes.
type Link<T> = Option<NonNull<Node<T>>>;

/// BTree struct
pub struct BTree<T> {
    root: Link<T>,
    size: usize,
    _marker: PhantomData<T>,
}

pub struct Iter<'a, T> {
    elems: Vec<&'a T>,
    current_idx: usize,
}

impl<T> Node<T> {
    /// Create new node.
    fn new(left: Link<T>, right: Link<T>, elem: T) -> NonNull<Node<T>> {
        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node { left, right, elem })))
        }
    }
}

impl<T: Ord> BTree<T> {
    /// Creates a new BinaryTree struct with no elements.
    pub fn new() -> Self {
        BTree {
            root: None,
            size: 0,
            _marker: PhantomData,
        }
    }

    /// Returns BinaryTree size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns whether the BinaryTree has no values.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }


    /// Insert a new node
    pub fn insert(&mut self, elem: T) {
        unsafe { self.root = self.insert_recursive(self.root, elem); }
    }

    /// Recursive function to insert a new node into de BTree.
    unsafe fn insert_recursive(&mut self, mut current: Link<T>, elem: T) -> Link<T> {
        if let Some(node) = current {
            unsafe {
                if elem < (*node.as_ptr()).elem {
                    (*node.as_ptr()).left = self.insert_recursive((*node.as_ptr()).left, elem);
                } else if elem > (*node.as_ptr()).elem {
                    (*node.as_ptr()).right = self.insert_recursive((*node.as_ptr()).right, elem);
                }
            }
        } else {
            let new_node = Some(Node::new(None, None, elem));
            current = new_node;
            self.size += 1
        }
        current
    }

    /// Returns `true` if the node with value is elem is on the BTree
    /// making use of [`BTree::search_recursive`].
    pub fn contains(&self, elem: &T) -> bool {
        unsafe {
            self.search_recursive(self.root, elem)
        }
    }


    unsafe fn search_recursive(&self, current: Link<T>, elem: &T) -> bool {
        match current {
            None => false,

            Some(node) => {
                unsafe {
                    // You could also take a reference of &(*node.as_ptr()).elem
                    // and compare it with elem which is a &T.
                    if *elem < (*node.as_ptr()).elem {
                        self.search_recursive((*node.as_ptr()).left, elem)
                    } else if *elem > (*node.as_ptr()).elem {
                        self.search_recursive((*node.as_ptr()).right, elem)
                    } else {
                        true
                    }
                }
            }
        }
    }
    

    // Returns a pointer to the parent node of the node that contains the
    /// minimum value in the given subtree. Used for searching inorder successors.
    unsafe fn min_value_parent_node(&self, node: NonNull<Node<T>>) -> Link<T> {
        unsafe {
            match (*node.as_ptr()).left {
                None => None,
                
                Some(node_left) => match (*node_left.as_ptr()).left {
                    None => Some(node),
                    Some(_) => self.min_value_parent_node(node_left),
                }
            }
        }
    }

    /// Removes `elem` from the BTree.
    pub fn remove(&mut self, elem: &T) {
        unsafe {
            self.root = self.remove_recursive(self.root, elem)
        }
    }

    /// BTree remove algorithm
    unsafe fn remove_recursive(&mut self, current: Link<T>, elem: &T) -> Link<T> {
        // Node not found
        if current.is_none() { return None; }

        // Search
        let node = current.unwrap();
        unsafe {
            if *elem < (*node.as_ptr()).elem {
                (*node.as_ptr()).left = self.remove_recursive((*node.as_ptr()).left, elem);
                return current;
            }
            
            if *elem > (*node.as_ptr()).elem {
                (*node.as_ptr()).right = self.remove_recursive((*node.as_ptr()).right, elem);
                return current;
            }


            // We found de Node.
            self.size -= 1;

            // Case 1: Node has only one child or None
            let mut replacement = None;
            if (*node.as_ptr()).left.is_none() {
                replacement = Some((*node.as_ptr()).right);
            } else if (*node.as_ptr()).right.is_none() {
                replacement = Some((*node.as_ptr()).left);
            }

            if replacement.is_some() {
                drop(Box::from_raw(node.as_ptr()));
                return replacement.unwrap();
            }

            // Case 2: Node has two children
            let node_to_drop;

            if let Some(parent) = self.min_value_parent_node((*node.as_ptr()).right.unwrap()) {
                node_to_drop = (*parent.as_ptr()).left.unwrap();
                let left = ptr::read(node_to_drop.as_ptr());
                (*node.as_ptr()).elem = left.elem;
                (*parent.as_ptr()).left = left.right
            } else {
                node_to_drop = (*node.as_ptr()).right.unwrap();
                let right = ptr::read(node_to_drop.as_ptr());
                (*node.as_ptr()).elem = right.elem;
                (*node.as_ptr()).right = right.right;
            }
            drop(Box::from_raw(node_to_drop.as_ptr()));
        }
        current 
    }
}

impl<T> BTree<T> {
    unsafe fn push_inorder(&self, current: Link<T>, elems: &mut Vec<&T>) {
        unsafe {
            if let Some(node) = current {
                self.push_inorder((*node.as_ptr()).left, elems);
                elems.push(&(*node.as_ptr()).elem);
                self.push_inorder((*node.as_ptr()).right, elems);
                
            }
        }
    }

    pub fn iter(&self) -> Iter<T> {
        let mut elems = Vec::with_capacity(self.size);

        unsafe {
            self.push_inorder(self.root, &mut elems);
        }

        Iter {
            elems,
            current_idx: 0,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx == self.elems.len() {
            return None;
        }

        let elem = self.elems[self.current_idx];

        self.current_idx += 1;
    
        Some(elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.elems.len() - self.current_idx;

        (remaining, Some(remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::BTree;

    fn tree_values() -> Vec<i32> {
        vec![40, 20, 60, 10, 30, 25, 35, 50, 45, 70, 80, 75]
    }

    #[test]
    fn test_insert() {
        let numbers = tree_values();

        let mut tree = BTree::new();

        tree.insert(numbers[0]);
        assert!(tree.contains(&numbers[0]));

        tree.insert(numbers[1]);
        assert!(tree.contains(&numbers[1]));

        tree.insert(numbers[2]);
        assert!(tree.contains(&numbers[2]));

        assert_eq!(tree.size(), 3);

        for n in &numbers[3..] {
            tree.insert(*n);
        }

        for n in &numbers {
            assert!(tree.contains(n));
        }

        assert_eq!(tree.size(), numbers.len());
    }

    #[test]
    fn test_remove() {
        let numbers = tree_values();

        let mut tree = BTree::new();

        for n in &numbers {
            tree.insert(*n);
        }

        // Node with no children
        tree.remove(&75);
        assert!(!tree.contains(&75));

        // Node with one child to the right
        tree.remove(&70);
        assert!(!tree.contains(&70));
        assert!(tree.contains(&80));

        // Node with one child to the left
        tree.remove(&50);
        assert!(!tree.contains(&50));
        assert!(tree.contains(&45));

        // Node with two children
        tree.remove(&20);
        assert!(!tree.contains(&20));
        assert!(tree.contains(&10));
        assert!(tree.contains(&30));

        // Root
        tree.remove(&40);
        assert!(!tree.contains(&40));

        // Check remaining values
        assert!(tree.contains(&80));
        assert!(tree.contains(&60));
        assert!(tree.contains(&35));
        assert!(tree.contains(&45));
        assert!(tree.contains(&25));
        assert!(tree.contains(&30));
        assert!(tree.contains(&10));
    }

    #[test]
    fn test_iter() {
        let mut values = tree_values();

        let mut tree = BTree::new();

        for value in values.iter() {
            tree.insert(*value);
        }

        let mut iter = tree.iter();

        values.sort();

        for value in values.iter() {
            let tree_value = iter.next();
            assert!(tree_value.is_some());
            assert_eq!(value, tree_value.unwrap());
        }

        assert!(iter.next().is_none());
    }
}