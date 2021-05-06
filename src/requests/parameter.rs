use crate::Service;
use std::{any::Any, fmt::Debug};

/// A parameter for configuring requested services.
pub trait RequestParameter: Service + Debug {
    /// Clones this parameter into a boxed trait object.
    fn clone_dyn(&self) -> Box<dyn RequestParameter>;

    /// Casts this parameter into a [`&dyn Any`](Any), making it easier to
    /// downcast into other types.
    fn as_any(&self) -> &dyn Any;
}

impl<T: Service + Debug + Clone> RequestParameter for T {
    fn clone_dyn(&self) -> Box<dyn RequestParameter> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl dyn RequestParameter {
    /// Tries to downcast this request parameter to a concrete type.
    pub fn downcast_ref<T: RequestParameter>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
}

impl Clone for Box<dyn RequestParameter> {
    fn clone(&self) -> Self {
        self.as_ref().clone_dyn()
    }
}
