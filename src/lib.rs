mod builder;
mod recyclable;

pub use builder::{builder, PoolBuilder, Supplier};
pub use recyclable::Recyclable;

use crossbeam::queue::SegQueue;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

pub trait InitializeWith<T> {
    fn initialize_with(&mut self, with: T);
}

pub struct Pool<T>
where
    T: Recyclable,
{
    settings: PoolBuilder<T>,
    values: SegQueue<T>,
}

impl<T> Pool<T>
where
    T: Recyclable,
{
    pub fn with_size(size: usize) -> Pool<T> {
        builder().with_starting_size(size).build()
    }

    pub fn get(&self) -> Recycled<T> {
        let value = self.values.pop().unwrap_or_else(|_| self.create());

        Recycled {
            value: ManuallyDrop::new(value),
            pool: self,
        }
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    fn create(&self) -> T {
        if let Some(supplier) = self.settings.supplier.as_ref() {
            supplier()
        } else {
            T::new()
        }
    }

    fn return_value(&self, mut value: T) {
        value.recycle();
        self.values.push(value)
    }
}

pub struct Recycled<'a, T>
where
    T: Recyclable,
{
    value: ManuallyDrop<T>,
    pool: &'a Pool<T>,
}

impl<'a, T> Recycled<'a, T> where T: Recyclable {}

impl<'a, T> Drop for Recycled<'a, T>
where
    T: Recyclable,
{
    fn drop(&mut self) {
        // Return value to pool.

        let value = unsafe {
            // Safe because the value is wrapped in ManuallyDrop,
            // so the uninitialized memory won't be read from.
            std::mem::replace(&mut self.value, MaybeUninit::uninit().assume_init())
        };
        let value = ManuallyDrop::into_inner(value);

        self.pool.return_value(value);
    }
}

impl<'a, T> AsRef<T> for Recycled<'a, T>
where
    T: Recyclable,
{
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<'a, T> AsMut<T> for Recycled<'a, T>
where
    T: Recyclable,
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<'a, T> Deref for Recycled<'a, T>
where
    T: Recyclable,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T> DerefMut for Recycled<'a, T>
where
    T: Recyclable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'a, T> Display for Recycled<'a, T>
where
    T: Recyclable + Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.as_ref())
    }
}

impl<'a, T> Debug for Recycled<'a, T>
where
    T: Recyclable + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.as_ref())
    }
}

impl<'a, T> PartialEq<T> for Recycled<'a, T>
where
    T: Recyclable + PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl<'a, T> PartialOrd<T> for Recycled<'a, T>
where
    T: Recyclable + PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.as_ref().partial_cmp(other)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use static_assertions::*;

    #[test]
    fn test_pool_send_and_sync() {
        assert_impl_all!(Pool<String>, Send, Sync);
    }

    #[test]
    fn test_builder() {
        let pool: Pool<String> = builder().with_starting_size(100).build();

        assert_eq!(pool.size(), 100);

        let value = pool.get();
        assert_eq!(pool.size(), 99);

        assert_eq!(*value, "");

        drop(value);

        assert_eq!(pool.size(), 100);
    }

    #[test]
    fn test_supplier() {
        let pool: Pool<String> = builder()
            .with_starting_size(4)
            .with_supplier(|| String::from("test"))
            .build();

        let mut value = pool.get();
        assert_eq!(*value, "test");

        value.push_str("bla");
        assert_eq!(*value, "testbla");
        drop(value);
    }
}
