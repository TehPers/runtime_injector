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

/// Marks a trait as being an interface for many other types. This means that
/// a request for the given trait can resolve to any of the types indicated by
/// this macro invocation.
///
/// With the "arc" feature enabled, the trait must be a subtrait of [`Send`]
/// and [`Sync`]. This is necessary to allow the service pointers to be
/// downcasted. If the "rc" feature is enabled, this is not required.
/// Additionally, instances of the trait must have a `'static` lifetime. This
/// can be done easily by making your interface a subtrait of [`Service`].
///
/// ## Example
///
/// ```
/// use runtime_injector::{interface, Service};
///
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

            fn should_provide(
                _provider: &dyn $crate::Provider<Interface = Self::Interface>,
            ) -> bool {
                true
            }

            fn from_provided(
                provided: Svc<Self::Interface>,
            ) -> InjectResult<Svc<Self>> {
                Ok(provided)
            }

            fn from_provided_owned(
                provided: Box<Self::Interface>,
            ) -> InjectResult<Box<Self>> {
                Ok(provided)
            }
        }
    };
}
