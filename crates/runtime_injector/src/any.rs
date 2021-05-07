use std::any::Any;

/// Defines a conversion for a type into a [`dyn Any`](Any) trait object.
pub trait AsAny: Any {
    /// Converts this reference into a trait object reference.
    fn as_any(&self) -> &dyn Any;

    /// Converts this reference into a mutable trait object reference.
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
