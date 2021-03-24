use crate::{InjectError, InjectResult, Injector, Service, ServiceInfo, Svc};
use std::any::Any;

/// Indicates that a type can resolve services. The most basic implementation
/// of this trait is that each sized service type can resolve itself. This is
/// done by requesting the exact implementation of itself from the injector.
/// However, the injector cannot provide exact implementations for dynamic
/// types (`dyn Trait`). For this reason, any interfaces using traits must be
/// declared explicitly before use. This trait should usually be implemented
/// automatically by the `interface!` macro.
pub trait Interface: Any {
    /// Attempts to resolve a service which implements this interface. If an
    /// implementation type is provided, this **should** attempt to return an
    /// instance of that type. If it cannot, then this should return an error.
    /// However, it is not unsound to return the wrong type. It is only unsafe
    /// for code to rely on that exact type being returned in an unsafe manner.
    fn resolve(
        injector: &mut Injector,
        implementation: Option<ServiceInfo>,
    ) -> InjectResult<Svc<Self>>;
}

impl<T: Service> Interface for T {
    fn resolve(
        injector: &mut Injector,
        implementation: Option<ServiceInfo>,
    ) -> InjectResult<Svc<Self>> {
        if let Some(implementation) = implementation {
            let service_info = ServiceInfo::of::<Self>();
            if service_info != implementation {
                return Err(InjectError::InvalidImplementation {
                    service_info,
                    implementation,
                });
            }
        }

        injector.get_exact()
    }
}

/// Marker trait that indicates that a type is an interface for another type.
/// Each sized type is an interface for itself, and each `dyn Trait` is an
/// interface for the types that it can resolve. This trait should usually be
/// implemented automatically by the `interface!` macro, and is strictly used
/// to enforce stronger type checking when assigning implementations for
/// interfaces.
pub trait InterfaceFor<T: Service>: Interface {}
impl<T: Service> InterfaceFor<T> for T {}

/// Marks a trait as being an interface for many other types. This means that
/// a request for the given trait can resolve to any of the types indicated by
/// this macro invocation.
///
/// # Example
/// ```
/// use runtime_injector::interface;
///
/// struct Bar;
/// #[cfg(test)]
/// struct MockBar;
///
/// trait Foo {}
/// impl Foo for Bar {}
/// #[cfg(test)]
/// impl Foo for MockBar {}
///
/// // Requests for `dyn Foo` can resolve to either `Bar` or, in a test run,
/// // `MockBar`. Note that attributes are allowed on each of the listed types.
/// interface!(
///     Foo = [
///         Bar,
///         #[cfg(test)]
///         MockBar,
///     ]
/// );
/// ```
#[macro_export]
macro_rules! interface {
    ($trait:tt = [$($(#[$attr:meta])* $impl:ty),* $(,)?]) => {
        impl $crate::Interface for dyn $trait {
            fn resolve(
                injector: &mut $crate::Injector,
                implementation: Option<$crate::ServiceInfo>,
            ) -> $crate::InjectResult<$crate::Svc<Self>> {
                match implementation {
                    $(
                        $(#[$attr])*
                        Some(implementation) if implementation == $crate::ServiceInfo::of::<$impl>() => {
                            Ok(injector.get::<$impl>()? as $crate::Svc<Self>)
                        }
                    ),*
                    Some(implementation) => {
                        Err($crate::InjectError::InvalidImplementation {
                            service_info: $crate::ServiceInfo::of::<Self>(),
                            implementation,
                        })
                    }
                    None => Err($crate::InjectError::MissingProvider { service_info: $crate::ServiceInfo::of::<Self>() })
                }
            }
        }

        $(
            $(#[$attr])*
            impl $crate::InterfaceFor<$impl> for dyn $trait {}
        )*
    };
}
