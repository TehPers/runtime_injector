use crate::{
    InjectError, InjectResult, Injector, Interface, ServiceInfo, Services, Svc,
};

/// A request to an injector.
pub trait Request: Sized {
    /// Performs the request to the injector.
    fn request(injector: &Injector) -> InjectResult<Self>;
}

impl Request for Injector {
    fn request(injector: &Injector) -> InjectResult<Self> {
        Ok(injector.clone())
    }
}

impl<I: ?Sized + Interface> Request for Svc<I> {
    fn request(injector: &Injector) -> InjectResult<Self> {
        let mut services: Services<I> = injector.get_service()?;
        if services.len() > 1 {
            Err(InjectError::MultipleProviders {
                service_info: ServiceInfo::of::<I>(),
                providers: services.len(),
            })
        } else {
            let service = services.get_all().next().ok_or(
                InjectError::MissingProvider {
                    service_info: ServiceInfo::of::<I>(),
                },
            )??;

            Ok(service)
        }
    }
}

impl<I: ?Sized + Interface> Request for Services<I> {
    fn request(injector: &Injector) -> InjectResult<Self> {
        injector.get_service()
    }
}

impl<I: ?Sized + Interface> Request for Vec<Svc<I>> {
    fn request(injector: &Injector) -> InjectResult<Self> {
        let mut impls: Services<I> = injector.get()?;
        impls.get_all().collect()
    }
}

impl<I: ?Sized + Interface> Request for Option<Svc<I>> {
    fn request(injector: &Injector) -> InjectResult<Self> {
        match injector.get() {
            Ok(response) => Ok(Some(response)),
            Err(InjectError::MissingProvider { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}
