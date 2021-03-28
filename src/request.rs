use crate::{InjectError, InjectResult, Injector, Interface, ServiceInfo, Svc};

pub trait Request: Sized {
    /// Performs the request to the container.
    fn request(injector: &mut Injector) -> InjectResult<Self>;
}

impl<I: ?Sized + Interface> Request for Svc<I> {
    fn request(injector: &mut Injector) -> InjectResult<Self> {
        let implementation =
            injector.get_implementation(ServiceInfo::of::<I>());
        I::resolve(injector, implementation)
    }
}

impl<I: ?Sized + Interface> Request for Option<Svc<I>> {
    fn request(injector: &mut Injector) -> InjectResult<Self> {
        match injector.get() {
            Ok(response) => Ok(Some(response)),
            Err(InjectError::MissingProvider { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}
