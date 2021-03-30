use crate::{InjectError, InjectResult, Injector, Interface, ServiceInfo, Svc};

/// A request to an injector.
pub trait Request: Sized {
    /// Performs the request to the injector.
    fn request(injector: &Injector) -> InjectResult<Self>;
}

impl<I: ?Sized + Interface> Request for Svc<I> {
    fn request(injector: &Injector) -> InjectResult<Self> {
        let implementation =
            injector.get_implementation(ServiceInfo::of::<I>());
        I::resolve(injector, implementation)
    }
}

impl<R: Request> Request for Option<R> {
    fn request(injector: &Injector) -> InjectResult<Self> {
        match injector.get() {
            Ok(response) => Ok(Some(response)),
            Err(InjectError::MissingProvider { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

impl Request for Injector {
    fn request(injector: &Injector) -> InjectResult<Self> {
        Ok(injector.clone())
    }
}
