use crate::{
    InjectResult, Injector, Request, RequestInfo, Service, ServiceInfo,
};

/// A factory for creating instances of a service. All functions of arity 12 or
/// less are automatically service factories if the arguments to that function
/// are valid service requests and the return value is a valid service type.
///
/// ```
/// use runtime_injector::{ServiceFactory, Injector, Svc, RequestInfo};
///
/// struct Foo;
/// struct Bar;
///
/// # fn _no_run() {
/// fn factory(foo: Svc<Foo>) -> Bar { todo!() }
/// let injector: Injector = todo!();
/// factory.invoke(&injector, RequestInfo::new());
/// # }
/// ```
///
/// # Type parameters
/// * `D` - Dependencies of this service as a tuple.
/// * `R` - Resulting service from invoking this service factory.
pub trait ServiceFactory<D, R>: 'static
where
    R: Service,
{
    /// Invokes this service factory, creating an instance of the service.
    fn invoke(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<R>;
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
        impl <F, R $(, $type_name)*> ServiceFactory<($($type_name,)*), R> for F
        where
            F: 'static + FnMut($($type_name),*) -> R,
            R: Service,
            $($type_name: Request,)*
        {
            #[allow(unused_variables, unused_mut, unused_assignments, non_snake_case)]
            fn invoke(&mut self, injector: &Injector, request_info: RequestInfo) -> InjectResult<R> {
                let request_info = request_info.with_request(ServiceInfo::of::<R>());
                let result = self($(
                    match <$type_name as Request>::request(&injector, request_info.clone()) {
                        Ok(dependency) => dependency,
                        Err($crate::InjectError::MissingProvider { service_info }) => {
                            return Err($crate::InjectError::MissingDependency {
                                dependency_info: service_info,
                                service_info: $crate::ServiceInfo::of::<R>(),
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
