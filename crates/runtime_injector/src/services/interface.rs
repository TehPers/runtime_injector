use crate::{
    DynSvc, InjectError, InjectResult, OwnedDynSvc, Service, ServiceInfo, Svc,
};

/// Indicates functionality that can be implemented.
///
/// For example, each [`Sized`] [`Service`] type is an interface that
/// implements itself. This is done by requesting instances of itself from the
/// injector. However, the injector cannot provide instances of dynamic types
/// (`dyn Trait`) automatically because they are unsized. For this reason, any
/// interfaces using traits must be declared explicitly before use. This trait
/// should usually be implemented by the [`interface!`] macro in the same
/// module the trait was declared in.
///
/// Since implementations of interfaces must be services, interfaces should be
/// declared with a supertrait of [`Service`]. This will ensure that the
/// implementors can be cast to [`dyn Any`](std::any::Any), and with the "arc"
/// feature enabled, those implementors are also [`Send`] + [`Sync`].
/// Additionally, trait interfaces must be convertible to trait objects so they
/// can be used behind service pointers. You can read more about trait objects
/// [here](https://doc.rust-lang.org/book/ch17-02-trait-objects.html).
///
/// See the documentation for the [`interface!`] macro for more information.
pub trait Interface: Service {
    /// Downcasts a dynamic service pointer into a service pointer of this
    /// interface type.
    fn downcast(service: DynSvc) -> InjectResult<Svc<Self>>;

    /// Downcasts an owned dynamic service pointer into an owned service
    /// pointer of this interface type.
    fn downcast_owned(service: OwnedDynSvc) -> InjectResult<Box<Self>>;
}

impl<T: Service> Interface for T {
    fn downcast(service: DynSvc) -> InjectResult<Svc<Self>> {
        service
            .downcast()
            .map_err(|_| InjectError::InvalidProvider {
                service_info: ServiceInfo::of::<Self>(),
            })
    }

    fn downcast_owned(service: OwnedDynSvc) -> InjectResult<Box<Self>> {
        service
            .downcast()
            .map_err(|_| InjectError::InvalidProvider {
                service_info: ServiceInfo::of::<Self>(),
            })
    }
}

/// Marker trait that indicates that a type is an interface for another type.
///
/// Each sized type is an interface for itself, and each `dyn Trait` is an
/// interface for the types that it can resolve. This trait should usually be
/// implemented by the [`interface!`] macro, and is strictly used to enforce
/// stronger type checking when assigning implementations for interfaces.
pub trait InterfaceFor<T: Service>: Interface {}
impl<T: Service> InterfaceFor<T> for T {}

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
/// // `MockBar`. Note that attributes are allowed on each of the listed types.
/// interface! {
///     Foo = [
///         Bar,
///         #[cfg(test)]
///         MockBar,
///     ]
/// };
/// ```
#[macro_export]
macro_rules! interface {
    {$trait:tt = [$($(#[$($attr:meta),*])* $impl:ty),* $(,)?]} => {
        impl $crate::Interface for dyn $trait {
            #[allow(unused_assignments)]
            fn downcast(mut service: $crate::DynSvc) -> $crate::InjectResult<$crate::Svc<Self>> {
                $(
                    $(#[$($attr),*])*
                    match service.downcast::<$impl>() {
                        Ok(downcasted) => return Ok(downcasted as $crate::Svc<Self>),
                        Err(input) => service = input,
                    }
                )*

                Err($crate::InjectError::MissingProvider { service_info: $crate::ServiceInfo::of::<Self>() })
            }

            #[allow(unused_assignments)]
            fn downcast_owned(mut service: $crate::OwnedDynSvc) -> $crate::InjectResult<::std::boxed::Box<Self>> {
                $(
                    $(#[$($attr),*])*
                    match service.downcast::<$impl>() {
                        Ok(downcasted) => return Ok(downcasted as ::std::boxed::Box<Self>),
                        Err(input) => service = input,
                    }
                )*

                Err($crate::InjectError::MissingProvider { service_info: $crate::ServiceInfo::of::<Self>() })
            }
        }

        $(
            $(#[$($attr),*])*
            impl $crate::InterfaceFor<$impl> for dyn $trait {}
        )*
    };
}
