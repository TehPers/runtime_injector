use crate::{
    InjectError, InjectResult, Injector, MappedServiceFactory, Request,
    RequestInfo, Service, ServiceInfo,
};
use std::any::Any;

/// A factory for creating instances of a service. All functions of arity 12 or
/// less are automatically service factories if the arguments to that function
/// are valid service requests and the return value is a valid service type.
///
/// ## Type parameters
/// * `D` - Tuple of this service's dependencies.
///
/// ## Example
///
/// ```
/// use runtime_injector::{Injector, RequestInfo, ServiceFactory, Svc};
///
/// struct Foo;
/// struct Bar(Svc<Foo>);
///
/// fn factory(foo: Svc<Foo>) -> Bar {
///     Bar(foo)
/// }
/// let injector = Injector::default();
/// factory.invoke(&injector, &RequestInfo::new());
/// ```
pub trait ServiceFactory<D> {
    /// The resulting service from invoking this service factory.
    type Result: Any;

    /// Invokes this service factory, creating an instance of the service.
    fn invoke(
        &self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Self::Result>;

    fn map<R, F>(self, f: F) -> MappedServiceFactory<R, Self, D, F>
    where
        Self: Sized,
        R: Service,
        F: Fn(Self::Result) -> R,
    {
        MappedServiceFactory::new(self, f)
    }
}

macro_rules! impl_provider_function {
    () => {
        impl_provider_function!(@impl ());
    };
    ($first:ident $(, $rest:ident)*) => {
        impl_provider_function!(@impl ($first $(, $rest)*));
        impl_provider_function!($($rest),*);
    };
    (@impl ($($type_name:ident),*)) => {
        impl<F, R $(, $type_name)*> ServiceFactory<($($type_name,)*)> for F
        where
            F: Fn($($type_name),*) -> R,
            R: Any,
            $($type_name: Request,)*
        {
            type Result = F::Output;

            #[allow(unused_variables, unused_mut, unused_assignments, non_snake_case)]
            fn invoke(
                &self,
                injector: &Injector,
                request_info: &RequestInfo
            ) -> InjectResult<Self::Result> {
                let result = self($(
                    match <$type_name as Request>::request(&injector, request_info) {
                        Ok(dependency) => dependency,
                        Err(InjectError::MissingProvider { service_info }) => {
                            return Err(InjectError::MissingDependency {
                                dependency_info: service_info,
                                service_info: ServiceInfo::of::<R>(),
                            })
                        },
                        Err(error) => return Err(error),
                    }
                ),*);
                Ok(result)
            }
        }
    };
}

impl_provider_function!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
