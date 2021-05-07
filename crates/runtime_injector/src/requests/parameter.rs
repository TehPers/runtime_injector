use crate::{AsAny, Service};
use std::fmt::Debug;

/// A parameter for configuring requested services.
pub trait RequestParameter: Service + Debug + AsAny {
    /// Clones this parameter into a boxed trait object.
    fn clone_dyn(&self) -> Box<dyn RequestParameter>;
}

impl<T: Service + Debug + Clone + AsAny> RequestParameter for T {
    fn clone_dyn(&self) -> Box<dyn RequestParameter> {
        Box::new(self.clone())
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
