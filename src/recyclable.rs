use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;

pub trait Recyclable: Send {
    fn new() -> Self
    where
        Self: Sized;
    fn recycle(&mut self);
}

// Recyclable implementations
impl Recyclable for String {
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<T> Recyclable for Vec<T>
where
    T: Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<T> Recyclable for VecDeque<T>
where
    T: Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<T> Recyclable for LinkedList<T>
where
    T: Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<K, V> Recyclable for HashMap<K, V>
where
    K: Eq + Hash + Send,
    V: Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<T> Recyclable for HashSet<T>
where
    T: Eq + Hash + Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<K, V> Recyclable for BTreeMap<K, V>
where
    K: Ord + Send,
    V: Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<T> Recyclable for BTreeSet<T>
where
    T: Ord + Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

impl<T> Recyclable for BinaryHeap<T>
where
    T: Ord + Send,
{
    fn new() -> Self {
        Self::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

#[cfg(feature = "hashbrown-impls")]
mod hashbrown {
    use crate::Recyclable;
    use hashbrown::{HashMap, HashSet};
    use std::hash::Hash;

    impl<K, V> Recyclable for HashMap<K, V>
    where
        K: Eq + Hash + Send,
        V: Send,
    {
        fn new() -> Self {
            Self::new()
        }

        fn recycle(&mut self) {
            self.clear()
        }
    }

    impl<T> Recyclable for HashSet<T>
    where
        T: Eq + Hash + Send,
    {
        fn new() -> Self {
            Self::new()
        }

        fn recycle(&mut self) {
            self.clear()
        }
    }
}
