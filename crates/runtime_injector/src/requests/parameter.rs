use std::any::Any;
use crate::{AsAny, Service};

/// A parameter for configuring requested services.
pub trait RequestParameter: Service + AsAny {
    /// Clones this parameter into a boxed trait object.
    fn clone_dyn(&self) -> Box<dyn RequestParameter>;
}

impl<T: Service + Clone + AsAny> RequestParameter for T {
    fn clone_dyn(&self) -> Box<dyn RequestParameter> {
        Box::new(self.clone())
    }
}

impl dyn RequestParameter {
    /// Tries to downcast this request parameter to a concrete reference type.
    pub fn downcast_ref<T: RequestParameter>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    /// Tries to downcast this request parameter to a concrete mutable reference type.
    pub fn downcast_mut<T: RequestParameter>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }

    /// Tries to downcast this request parameter to a concrete owned type.
    pub fn downcast<T: RequestParameter>(self: Box<Self>) -> Result<Box<T>, Box<dyn Any>> {
        self.into_any().downcast()
    }
}

impl Clone for Box<dyn RequestParameter> {
    fn clone(&self) -> Self {
        self.as_ref().clone_dyn()
    }
}
