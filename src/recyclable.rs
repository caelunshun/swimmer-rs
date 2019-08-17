use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;

macro_rules! num_recyclable_impl {
    ($num:ident) => {
        impl Recyclable for $num {
            fn new() -> Self {
                0
            }

            fn recycle(&mut self) {
                *self = 0
            }
        }
    };
}

/// Indicates that an object can be used
/// inside a `Pool`.
///
/// Types implementing this trait must be `Send`,
/// since the pool itself can be used across threads.
pub trait Recyclable: Send {
    /// Creates a new value of this type.
    fn new() -> Self
    where
        Self: Sized;

    /// Resets this object, allowing it to
    /// be reused in the future without retaining
    /// its old state.
    fn recycle(&mut self);
}

// Recyclable implementations
impl Recyclable for String {
    fn new() -> Self {
        String::new()
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
        Vec::new()
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
        VecDeque::new()
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
        LinkedList::new()
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
        HashMap::new()
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
        HashSet::new()
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
        BTreeMap::new()
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
        BTreeSet::new()
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
        BinaryHeap::new()
    }

    fn recycle(&mut self) {
        self.clear()
    }
}

num_recyclable_impl!(u8);
num_recyclable_impl!(u16);
num_recyclable_impl!(u32);
num_recyclable_impl!(u64);
num_recyclable_impl!(i8);
num_recyclable_impl!(i16);
num_recyclable_impl!(i32);
num_recyclable_impl!(i64);

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
            HashMap::new()
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
            HashSet::new()
        }

        fn recycle(&mut self) {
            self.clear()
        }
    }
}

#[cfg(feature = "smallvec-impls")]
mod smallvec {
    use crate::Recyclable;
    use smallvec::{Array, SmallVec};

    impl<T, A> Recyclable for SmallVec<A>
    where
        A: Array<Item = T>,
        T: Send,
    {
        fn new() -> Self {
            SmallVec::new()
        }

        fn recycle(&mut self) {
            self.clear()
        }
    }
}
