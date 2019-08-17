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
