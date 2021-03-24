use crate::{Dependencies, InjectResult, Injector, Interface, Service, Svc};

pub trait ProviderFunction<D, R>: 'static
where
    D: Dependencies,
    R: Service,
{
    fn invoke(&mut self, injector: &mut Injector) -> InjectResult<Svc<R>>;
}

macro_rules! impl_provider_function {
    () => {
        impl_provider_function!(@impl ());
    };
    ($first:ident $(, $rest:ident)*) => {
        // Assert that $n type names are given
        impl_provider_function!(@impl ($first $(, $rest)*));
        impl_provider_function!($($rest),*);
    };
    (@impl ($($type_name:ident),*)) => {
        impl <F, R $(, $type_name)*> ProviderFunction<($(Svc<$type_name>,)*), R> for F
        where
            F: 'static + FnMut($(Svc<$type_name>),*) -> R,
            R: Service,
            $($type_name: ?Sized + Interface,)*
        {
            #[allow(unused_variables, unused_mut, unused_assignments, non_snake_case)]
            fn invoke(&mut self, injector: &mut Injector) -> InjectResult<Svc<R>> {
                let result = self($(
                    match injector.get::<$type_name>() {
                        Ok(dependency) => dependency,
                        Err($crate::InjectError::MissingProvider { service_info }) => {
                            Err($crate::InjectError::MissingDependency {
                                dependency_info: service_info,
                                service_info: $crate::ServiceInfo::of::<R>(),
                            })?
                        },
                        err @ Err(_) => err?,
                    }
                ),*);
                Ok(Svc::new(result))
            }
        }
    };
}

impl_provider_function!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
