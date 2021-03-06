use crate::{
    InjectError, InjectResult, Injector, Interface, RequestInfo, ServiceInfo,
    Services, Svc,
};

/// A request to an injector.
///
/// ## Grouping requests
///
/// Requests can be grouped together by using tuples to make multiple requests
/// at once. Since there is a limit of 12 supported parameters/dependencies for
/// factories, tuples can also be used to get around that limitation.
///
/// ```
/// use runtime_injector::{Injector, IntoSingleton, Svc};
///
/// struct Bar;
/// struct Baz;
/// struct Foo(Svc<Bar>, Svc<Baz>);
///
/// impl Foo {
///     pub fn new((bar, baz): (Svc<Bar>, Svc<Baz>)) -> Self {
///         Foo(bar, baz)
///     }
/// }
///
/// let mut builder = Injector::builder();
/// builder.provide(Foo::new.singleton());
///
/// let _injector = builder.build();
/// ```
///
/// ## Owned service requests
///
/// Some services can be provided directly via owned pointers ([`Box<I>`]).
/// These services can be requested directly via [`Box<I>`], as a collection
/// via [`Vec<Box<I>>`](Vec<I>), or lazily via an iterator via [`Services<I>`].
///
/// ```
/// use runtime_injector::{define_module, Injector, IntoTransient, Svc};
///
/// #[derive(Default)]
/// struct Foo;
/// struct Bar(Box<Foo>);
///
/// let module = define_module! {
///     services = [
///         Foo::default.transient(),
///         Bar.transient(),
///     ]
/// };
///
/// let mut builder = Injector::builder();
/// builder.add_module(module);
///
/// let injector = builder.build();
/// let _bar: Box<Bar> = injector.get().unwrap();
/// ```
pub trait Request: Sized {
    /// Performs the request to the injector.
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self>;
}

/// Requests the injector used to resolve services.
impl Request for Injector {
    fn request(injector: &Injector, _info: &RequestInfo) -> InjectResult<Self> {
        Ok(injector.clone())
    }
}

/// Requests the information about the current request.
impl Request for RequestInfo {
    fn request(_injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        Ok(info.clone())
    }
}

/// Requests a service pointer to a service or interface. This request fails if
/// there is not exactly one implementation of the given interface.
impl<I: ?Sized + Interface> Request for Svc<I> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        let mut services: Services<I> = injector.get_with(info)?;
        if services.len() > 1 {
            Err(InjectError::MultipleProviders {
                service_info: ServiceInfo::of::<I>(),
                providers: services.len(),
            })
        } else {
            let service = services.get_all().next().transpose()?.ok_or(
                InjectError::MissingProvider {
                    service_info: ServiceInfo::of::<I>(),
                },
            )?;

            Ok(service)
        }
    }
}

/// Requests an owned pointer to a service or interface. Not all providers can
/// provide owned pointers to their service, so this may fail where [`Svc<T>`]
/// requests would otherwise succeed.
impl<I: ?Sized + Interface> Request for Box<I> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        let mut services: Services<I> = injector.get_with(info)?;
        if services.len() > 1 {
            Err(InjectError::MultipleProviders {
                service_info: ServiceInfo::of::<I>(),
                providers: services.len(),
            })
        } else {
            let service = services.get_all_owned().next().transpose()?.ok_or(
                InjectError::MissingProvider {
                    service_info: ServiceInfo::of::<I>(),
                },
            )?;

            Ok(service)
        }
    }
}

/// Lazily requests all the implementations of an interface.
impl<I: ?Sized + Interface> Request for Services<I> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        injector.get_service(info)
    }
}

/// Requests all the implementations of an interface. For sized types, this
/// will return at most one implementation. If no provider is registered for
/// the given interface, then this will return an empty [`Vec<T>`].
impl<I: ?Sized + Interface> Request for Vec<Svc<I>> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        let mut impls: Services<I> = injector.get_with(info)?;
        impls.get_all().collect()
    }
}

/// Requests all the implementations of an interface as owned service pointers.
/// If no provider is registered for the given interface, then this will return
/// an empty [`Vec<T>`]. If any provider cannot provide an owned service
/// pointer, then an error is returned instead.
impl<I: ?Sized + Interface> Request for Vec<Box<I>> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        let mut impls: Services<I> = injector.get_with(info)?;
        impls.get_all_owned().collect()
    }
}

/// Tries to request a service pointer for a service or interface. If no
/// provider has been registered for it, then returns `None`. This fails if
/// there are multiple implementations of the given interface.
impl<I: ?Sized + Interface> Request for Option<Svc<I>> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        match injector.get_with(info) {
            Ok(response) => Ok(Some(response)),
            Err(InjectError::MissingProvider { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

/// Tries to request an ownedservice pointer for a service or interface. If no
/// provider has been registered for it, then returns `None`. This fails if
/// there are multiple implementations of the given interface or if the service
/// cannot be provided via an owned service pointer.
impl<I: ?Sized + Interface> Request for Option<Box<I>> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        match injector.get_with(info) {
            Ok(response) => Ok(Some(response)),
            Err(InjectError::MissingProvider { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

macro_rules! impl_tuple_request {
    () => {
        impl_tuple_request!(@impl ());
    };
    ($first:ident $(, $rest:ident)*) => {
        impl_tuple_request!(@impl ($first $(, $rest)*));
        impl_tuple_request!($($rest),*);
    };
    (@impl ($($type_name:ident),*)) => {
        /// Performs multiple requests at once. This is useful for grouping
        /// together related requests.
        impl <$($type_name),*> Request for ($($type_name,)*)
        where
            $($type_name: Request,)*
        {
            #[allow(unused_variables)]
            fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
                let result = ($(injector.get_with::<$type_name>(info)?,)*);
                Ok(result)
            }
        }
    };
}

impl_tuple_request!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
