use crate::{Pool, Recyclable, init};
use thread_local::CachedThreadLocal;

/// Creates a new `PoolBuilder`, used
/// to initialize a `Pool`.
///
/// This function will use a default
/// starting size and no supplier function.
pub fn builder<T>() -> PoolBuilder<T>
where
    T: Recyclable,
{
    PoolBuilder::default()
}

/// A supplier function, used to initialize
/// new objects for a pool.
pub type Supplier<T> = dyn Fn() -> T + Send + Sync;

/// A pool builder, used to configure various
/// pool settings.
pub struct PoolBuilder<T: Recyclable> {
    pub(crate) starting_size: usize,
    pub(crate) supplier: Option<Box<Supplier<T>>>,
}

impl<T> PoolBuilder<T>
where
    T: Recyclable,
{
    /// See `Pool::with_size`.
    pub fn with_starting_size(mut self, starting_size: usize) -> Self {
        self.starting_size = starting_size;
        self
    }

    /// Uses the given closure for initializing
    /// new objects in the pool.
    pub fn with_supplier<S>(mut self, supplier: S) -> Self
    where
        S: Fn() -> T + Send + Sync + 'static,
    {
        self.supplier = Some(Box::new(supplier));
        self
    }

    /// Builds a pool using the configured settings.
    pub fn build(self) -> Pool<T> {
        let values = CachedThreadLocal::new();

        for _ in 0..self.starting_size {
            if let Some(supplier) = self.supplier.as_ref() {
                values.get_or(|| init()).borrow_mut().push(supplier())
            } else {
                values.get_or(|| init()).borrow_mut().push(T::new())
            }
        }

        Pool {
            values,
            settings: self,
        }
    }

    /// Builds a pool using the configured settings, and fill it with the given items.
    pub fn build_with(self, items: Vec<T>) -> Pool<T> {
        let values = CachedThreadLocal::new();

        let size = items.len();
        for itm in items {
            values.get_or(|| init()).borrow_mut().push(itm);
        }

        if size < self.starting_size {
            let remainder = self.starting_size - size;
            for _ in 0..remainder{
                if let Some(supplier) = self.supplier.as_ref() {
                    values.get_or(|| init()).borrow_mut().push(supplier())
                } else {
                    values.get_or(|| init()).borrow_mut().push(T::new())
                }
            }
        }

        Pool {
            values,
            settings: self,
        }
    }
}

impl<T> Default for PoolBuilder<T>
where
    T: Recyclable,
{
    fn default() -> Self {
        Self {
            starting_size: 0,
            supplier: None,
        }
    }
}
