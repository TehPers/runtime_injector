use std::any::Any;

/// Defines a conversion for a type into an [`dyn Any`](Any) trait object.
pub trait AsAny: Any {
    /// Converts `self` into a trait object.
    fn as_any(&self) -> &dyn Any;

    /// Converts `self` into a mutable trait object.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
