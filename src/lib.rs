#![forbid(missing_docs)]
#![doc(html_root_url = "https://docs.rs/swimmer/0.2.0")]

//! A thread-safe object pool for Rust.
//!
//! An object pool is used to reuse
//! objects without reallocating them.
//!  When an object is requested from
//! a pool, it is taken out of the pool; once
//! it is dropped, it is returned to the pool
//! and can be retrieved once more.
//!
//! The main type of this crate is the [`Pool`](struct.Pool.html)
//! struct, which implements a thread-safe object pool.
//! It can pool objects which implement [`Recyclable`](struct.Recyclable.html),
//! a trait which allows the pool to initialize and "recycle"
//! an object.
//!
//! The implementation of this is as follows:
//! * A pool is created using the [`builder`](fn.builder.html)
//! function. It is configured with an initial size.
//! * Upon creation of the pool, the pool initializes
//! `initial_size` values using `Recyclable`'s `new` function.
//! * When a value is requested from the pool, usually
//! using `Pool::get()`, a value is taken out of the internal
//! buffer. If there are no remaining values, a new object
//! is initialized using `Recyclable::new()`.
//! * The value can then be used by the caller.
//! * When the value is dropped, it is returned to the pool,
//! and future calls to `Pool::get()` may return the same object.
//!
//! To ensure that the object is cleaned, the pool calls `Recyclable::recycle()`
//! on the object before returning it to the pool. This function removes
//! any mutated state of the object, effectively "resetting" it. For
//! example, see the following sequence of events:
//! * A pool of vectors is initialized.
//! * A vector is retrieved from the pool, and some values are added to it.
//! * The vector is dropped and returned to the pool.
//!
//! Without resetting the vector, future calls to `Pool::get` could return
//! a vector containing those old elements; clearly, this is not desirable.
//! As a result, the `Recyclable` implementation for `Vec` clears the
//! vector when recycling.
//!
//! This crate is heavily based on the `lifeguard` crate, but
//! it is thread-safe, while `lifeguard` is not.
//!
//! # Thread safety
//! `Pool` is thread-safe, and it can be shared across threads
//! or used in a lazily-initialized static variable (see the examples).
//!
//! This is currently implemented by making the pool contain
//! a thread-local buffer for each thread, which has been proving
//! by benchmarks to be more than twice as performant as using
//! a locked `Vec` or `crossbeam::SegQueue`.
//!
//! # Supplier
//! In some cases, you may want to specify your own function
//! for initializing new objects rather than use the default
//! `Recyclable::new()` function. In this case, you can optionally
//! use `PoolBuilder::with_supplier()`, which will cause
//! the pool to use the provided closure to initialize
//! new values.
//!
//! For example, the `Recyclable` implementation for `Vec<T>`
//! allocates a vector with zero capacity, but you may want
//! to give the vector an initial capacity. In that case,
//! you can do this, for example:
//! ```
//! use swimmer::Pool;
//! let pool: Pool<Vec<u32>> = swimmer::builder()
//!     .with_supplier(|| Vec::with_capacity(128))
//!     .build();
//!
//! let vec = pool.get();
//! assert_eq!(vec.capacity(), 128);
//! ```
//!
//! Note, however, that the supplier function is only
//! called when the object is first initialized: it is
//! not used to recycle the object. This means that there
//! is currently no way to implement custom recycling
//! functionality.
//!
//! # Crate features
//! * `hashbrown-impls`: implements `Recyclable` for `hashbrown::HashMap` and
//! `hashbrown::HashSet`.
//! * `smallvec-impls`: implements `Recyclable` for `SmallVec`.
//!
//! # Examples
//! Basic usage:
//! ```
//! use swimmer::Pool;
//!
//! // Initialize a new pool, allocating
//! // 10 empty values to start
//! let pool: Pool<String> = swimmer::builder()
//!     .with_starting_size(10)
//!     .build();
//!
//! assert_eq!(pool.size(), 10);
//!
//! let mut string = pool.get();
//! assert_eq!(*string, ""); // Note that you need to dereference the string, since it is stored in a special smart pointer
//! string.push_str("test"); // Mutate the string
//!
//! // One object was taken from the pool,
//! // so its size is now 9
//! assert_eq!(pool.size(), 9);
//!
//! // Now, the string is returned to the pool
//! drop(string);
//!
//! assert_eq!(pool.size(), 10);
//!
//! // Get another string from the pool. This string
//! // could be the same one retrieved above, but
//! // since the string is cleared before returning
//! // into the pool, it is now empty. However, it
//! // retains any capacity which was allocated,
//! // which prevents additional allocations
//! // from occurring.
//! let another_string = pool.get();
//! assert_eq!(*another_string, "");
//! ```
//!
//! Implementing `Recyclable` on your own object:
//! ```
//! use swimmer::{Pool, Recyclable};
//!
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! impl Recyclable for Person {
//!     fn new() -> Self {
//!         Self {
//!             name: String::new(),
//!             age: 0,
//!         }
//!     }
//!
//!     fn recycle(&mut self) {
//!         // You are responsible for ensuring
//!         // that modified `Person`s get reset
//!         // before being returned to the pool.
//!         // Otherwise, the object could be put
//!         // back into the pool with its old state
//!         // still intact; this could cause weird behavior!
//!         self.name.clear();
//!         self.age = 0;
//!      }
//! }
//!
//! let pool: Pool<Person> = Pool::new();
//! let mut josh = pool.get();
//! josh.name.push_str("Josh"); // Since `recycle` empties the string, this will effectively set `name` to `Josh`
//! josh.age = 47;
//!
//! drop(josh); // Josh is returned to the pool and his name and age are reset
//!
//! // Now get a new person
//! let another_person = pool.get();
//! ```
//! Using a `Pool` object in a `lazy_static` variable,
//! allowing it to be used globally:
//! ```
//! use lazy_static::lazy_static;
//! use swimmer::Pool;
//!
//! lazy_static! {
//!     static ref POOL: Pool<String> = {
//!         Pool::new()
//!     };
//! }
//!
//! let value = POOL.get();
//! ```

mod builder;
#[allow(clippy::implicit_hasher)] // No way to initialize a hash map with generic hasher
mod recyclable;

pub use builder::{builder, PoolBuilder, Supplier};
pub use recyclable::Recyclable;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use thread_local::ThreadLocal;

/// A thread-safe object pool, used
/// to reuse objects without reallocating.
///
/// See the crate-level documentation for more information.
#[derive(Default)]
pub struct Pool<T>
where
    T: Recyclable,
{
    settings: PoolBuilder<T>,
    values: ThreadLocal<RefCell<Vec<T>>>,
}

impl<T> Pool<T>
where
    T: Recyclable,
{
    /// Creates a new pool with default settings.
    ///
    /// This is equivalent to `swimmer::builder().build()`.
    ///
    /// # Examples
    /// ```
    /// use swimmer::Pool;
    /// let pool: Pool<String> = Pool::new();
    /// // Use the pool...
    /// ```
    pub fn new() -> Pool<T> {
        builder().build()
    }

    /// Creates a new pool with the specified
    /// starting size. The pool will allocate
    /// `size` initial values and insert them into
    /// the pool.
    ///
    /// This is equivalent to `swimmer::builder().with_size(size).build()`.
    ///
    /// # Examples
    /// ```
    /// use swimmer::Pool;
    /// let pool: Pool<Vec<String>> = Pool::with_size(16);
    /// assert_eq!(pool.size(), 16);
    /// ```
    pub fn with_size(size: usize) -> Pool<T> {
        builder().with_starting_size(size).build()
    }

    /// Retrieves a value from the pool.
    ///
    /// The value
    /// is returned using a `Recycled` smart pointer
    /// which returns the object to the pool when dropped.
    ///
    /// # Examples
    /// ```
    /// use swimmer::Pool;
    /// let pool: Pool<String> = Pool::new();
    ///
    /// let string = pool.get();
    /// assert_eq!(*string, "");
    /// ```
    pub fn get(&self) -> Recycled<T> {
        let value = self.get_raw_value();

        Recycled {
            value: ManuallyDrop::new(value),
            pool: self,
        }
    }

    /// Returns the current size of the pool.
    ///
    /// When an object is removed from the pool,
    /// the size is decremented; when it is returned, the
    /// size is incremented.
    ///
    /// # Examples
    /// ```
    /// use swimmer::Pool;
    /// let pool: Pool<String> = Pool::with_size(16);
    ///
    /// assert_eq!(pool.size(), 16);
    ///
    /// let _string = pool.get();
    /// assert_eq!(pool.size(), 15);
    ///
    /// drop(_string);
    /// assert_eq!(pool.size(), 16);
    /// ```
    pub fn size(&self) -> usize {
        self.values.get_or(|| init()).borrow().len()
    }

    /// Attaches `value` to this pool, wrapping
    /// it in a smart pointer which will return the
    /// object into the pool when dropped.
    ///
    /// # Examples
    /// ```
    /// use swimmer::Pool;
    /// let pool: Pool<u64> = Pool::with_size(0);
    /// assert_eq!(pool.size(), 0);
    ///
    /// let ten = pool.attach(10);
    /// // `ten` is still borrowed from the pool,
    /// // so the size hasn't changed
    /// assert_eq!(pool.size(), 0);
    ///
    /// // When dropped, `ten` will be returned
    /// // back to the pool
    /// drop(ten);
    /// assert_eq!(pool.size(), 1);
    /// ```
    pub fn attach(&self, value: T) -> Recycled<T> {
        Recycled {
            value: ManuallyDrop::new(value),
            pool: self,
        }
    }

    /// Detatches a value from this pool.
    ///
    /// This is equivalent to `get`, except
    /// for that the object will **not** be returned
    /// to the pool when dropped—it will simply be dropped.
    ///
    /// # Examples
    /// ```
    /// use swimmer::Pool;
    /// let pool: Pool<String> = Pool::with_size(10);
    ///
    /// let detached_string = pool.detached();
    /// assert_eq!(pool.size(), 9);
    ///
    /// // When dropped, the string won't
    /// // be returned to the pool
    /// drop(detached_string);
    /// assert_eq!(pool.size(), 9);
    /// ```
    pub fn detached(&self) -> T {
        self.get_raw_value()
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
        self.values.get_or(|| init()).borrow_mut().push(value);
    }

    fn get_raw_value(&self) -> T {
        self.values
            .get_or(|| init())
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| self.create())
    }
}

fn init<T>() -> RefCell<Vec<T>> {
    RefCell::new(vec![])
}

/// A smart pointer which returns the contained
/// object to its pool once dropped.
///
/// Objects of this type are obtained using `Pool::get`.
pub struct Recycled<'a, T>
where
    T: Recyclable,
{
    value: ManuallyDrop<T>,
    pool: &'a Pool<T>,
}

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
        assert_impl_all!(Pool<String>: Send, Sync);
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
