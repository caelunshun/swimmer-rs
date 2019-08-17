use crate::{Pool, Recyclable};
use crossbeam::queue::SegQueue;

pub fn builder<T>() -> PoolBuilder<T>
where
    T: Recyclable,
{
    PoolBuilder {
        starting_size: 8,
        supplier: None,
    }
}

pub type Supplier<T> = dyn Fn() -> T + Send + Sync;

pub struct PoolBuilder<T: Recyclable> {
    pub(crate) starting_size: usize,
    pub(crate) supplier: Option<Box<Supplier<T>>>,
}

impl<T> PoolBuilder<T>
where
    T: Recyclable,
{
    pub fn with_starting_size(mut self, starting_size: usize) -> Self {
        self.starting_size = starting_size;
        self
    }

    pub fn with_supplier<S>(mut self, supplier: S) -> Self
    where
        S: Fn() -> T + Send + Sync + 'static,
    {
        self.supplier = Some(Box::new(supplier));
        self
    }

    pub fn build(self) -> Pool<T> {
        let values = SegQueue::new();

        for _ in 0..self.starting_size {
            if let Some(supplier) = self.supplier.as_ref() {
                values.push(supplier())
            } else {
                values.push(T::new())
            }
        }

        Pool {
            values,
            settings: self,
        }
    }
}
