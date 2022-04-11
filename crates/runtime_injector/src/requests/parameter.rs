use crate::Service;
use downcast_rs::impl_downcast;

/// A parameter for configuring requested services.
pub trait RequestParameter: Service {
    /// Clones this parameter into a boxed trait object.
    fn clone_dyn(&self) -> Box<dyn RequestParameter>;
}

#[cfg(feature = "arc")]
impl_downcast!(sync RequestParameter);

#[cfg(feature = "rc")]
impl_downcast!(RequestParameter);

impl<T: Service + Clone> RequestParameter for T {
    fn clone_dyn(&self) -> Box<dyn RequestParameter> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn RequestParameter> {
    fn clone(&self) -> Self {
        self.as_ref().clone_dyn()
    }
}
