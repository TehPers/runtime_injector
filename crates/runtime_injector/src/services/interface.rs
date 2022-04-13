use crate::{Service, Svc};

/// Implemented for trait objects.
pub trait Interface: Service {}

/// Marker trait that indicates that a type is an interface for another type.
///
/// Each `dyn Trait` is an interface for the types that it can resolve. This
/// trait should usually be implemented by the [`interface!`] macro, and is
/// primarily used to enforce stronger type checking when assigning
/// implementations for interfaces.
pub trait InterfaceFor<S>: Interface
where
    S: Service,
{
    #[doc(hidden)]
    fn from_svc(service: Svc<S>) -> Svc<Self>;

    #[doc(hidden)]
    fn from_owned_svc(service: Box<S>) -> Box<Self>;
}

/// Marks a trait as being an interface. This means that a request for the
/// given trait can resolve to services of any of the types that implement it.
/// Those services must be registered to this interface when building the
/// [`Injector`](crate::Injector).
///
/// The interface trait must be a subtrait of [`Service`]. This means that
/// implementors must have a static lifetime. If the "arc" feature is enabled,
/// they must also be [`Send`] + [`Sync`].
///
/// ## Example
///
/// ```
/// use runtime_injector::{
///     interface, Injector, IntoSingleton, Service, Svc, WithInterface,
/// };
///
/// #[derive(Default)]
/// struct Bar;
/// #[cfg(test)]
/// struct MockBar;
///
/// trait Foo: Service {}
/// impl Foo for Bar {}
/// #[cfg(test)]
/// impl Foo for MockBar {}
///
/// // Requests for `dyn Foo` can resolve to either `Bar` or, in a test run,
/// // `MockBar`.
/// interface!(Foo);
///
/// let mut builder = Injector::builder();
/// builder.provide(Bar::default.singleton().with_interface::<dyn Foo>());
///
/// let injector = builder.build();
/// let _bar: Svc<dyn Foo> = injector.get().unwrap();
/// ```
#[macro_export]
macro_rules! interface {
    ($interface:tt) => {
        impl $crate::Interface for dyn $interface {}

        impl<T: $interface> $crate::InterfaceFor<T> for dyn $interface {
            fn from_svc(service: $crate::Svc<T>) -> $crate::Svc<Self> {
                service
            }

            fn from_owned_svc(
                service: ::std::boxed::Box<T>,
            ) -> ::std::boxed::Box<Self> {
                service
            }
        }

        impl $crate::FromProvider for dyn $interface {
            type Interface = Self;

            const SERVICE_TYPE: $crate::ServiceType =
                $crate::ServiceType::Interface;

            fn should_provide(
                _provider: &dyn $crate::Provider<Interface = Self::Interface>,
            ) -> bool {
                true
            }

            fn from_provided(
                provided: $crate::Svc<Self::Interface>,
            ) -> $crate::InjectResult<$crate::Svc<Self>> {
                ::std::result::Result::Ok(provided)
            }

            fn from_provided_owned(
                provided: ::std::boxed::Box<Self::Interface>,
            ) -> $crate::InjectResult<::std::boxed::Box<Self>> {
                ::std::result::Result::Ok(provided)
            }
        }
    };
}
