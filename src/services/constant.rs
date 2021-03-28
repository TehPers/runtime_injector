use crate::{InjectResult, Injector, Service, Svc, TypedProvider};

/// A provider which returns a constant, predetermined value. Note that this is
/// technically a singleton service in that it does not recreate the value each
/// time it is requested.
pub struct ConstantProvider<R>
where
    R: Service,
{
    result: Svc<R>,
}

impl<R> ConstantProvider<R>
where
    R: Service,
{
    /// Creates a new `ConstantProvider` using a predetermined value.
    #[must_use]
    pub fn new(value: R) -> Self {
        ConstantProvider {
            result: Svc::new(value),
        }
    }
}

impl<R> TypedProvider for ConstantProvider<R>
where
    R: Service,
{
    type Result = R;

    fn provide_typed(
        &mut self,
        _injector: &mut Injector,
    ) -> InjectResult<Svc<Self::Result>> {
        Ok(self.result.clone())
    }
}

impl<T: Service> From<T> for ConstantProvider<T> {
    fn from(value: T) -> Self {
        constant(value)
    }
}

/// Create a service from a constant value. While the service itself will never
/// be exposed through a mutable reference, if it supports interior mutability,
/// its fields still can be mutated. Since the provider created with this
/// function doesn't recreate the value each time it's requested, state can be
/// stored in this manner.
///
/// # Example
///
/// ```
/// use runtime_injector::{Injector, Svc, constant};
///
/// let mut builder = Injector::builder();
/// builder.provide(constant(8i32));
///
/// let mut injector = builder.build();
/// let value: Svc<i32> = injector.get().unwrap();
///
/// assert_eq!(8, *value);
/// ```
///
/// # Interior mutability
///
/// One use case for constant values is to create a mutex to lock static,
/// synchronized values that can be accessed from any service. For instance,
/// suppose you wanted to create a counter to keep track of how many instances
/// of a service you created:
///
/// ```
/// use runtime_injector::{Injector, Svc, constant, IntoTransient};
/// use std::sync::Mutex;
///
/// struct Foo;
/// impl Foo {
///     pub fn new(counter: Svc<Mutex<i32>>) -> Self {
///         let mut counter = counter.lock().unwrap();
///         *counter += 1;
///         Foo
///     }
/// }
///
/// let mut builder = Injector::builder();
/// builder.provide(Foo::new.transient());
/// builder.provide(constant(Mutex::new(0i32)));
///
/// let mut injector = builder.build();
/// let foo: Svc<Foo> = injector.get().unwrap();
/// let value: Svc<Mutex<i32>> = injector.get().unwrap();
///
/// assert_eq!(1, *value.lock().unwrap());
/// ```
pub fn constant<T: Service>(value: T) -> ConstantProvider<T> {
    ConstantProvider::new(value)
}
