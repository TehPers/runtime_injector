use crate::{
    InjectError, InjectResult, Injector, RequestInfo, Service, ServiceInfo,
    Svc, TypedProvider,
};

/// A [`TypedProvider`] which conditionally provides its service. If the
/// condition is not met, then the provider is skipped during resolution.
///
/// See the [docs for `WithCondition`](crate::WithCondition) for more
/// information.
pub struct ConditionalProvider<P, F>
where
    P: TypedProvider,
    F: Service + Fn(&Injector, &RequestInfo) -> bool,
{
    inner: P,
    condition: F,
}

impl<P, F> TypedProvider for ConditionalProvider<P, F>
where
    P: TypedProvider,
    F: Service + Fn(&Injector, &RequestInfo) -> bool,
{
    type Result = P::Result;

    #[inline]
    fn provide_typed(
        &mut self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Svc<Self::Result>> {
        if (self.condition)(injector, request_info) {
            self.inner.provide_typed(injector, request_info)
        } else {
            Err(InjectError::ConditionsNotMet {
                service_info: ServiceInfo::of::<Self::Result>(),
            })
        }
    }

    #[inline]
    fn provide_owned_typed(
        &mut self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Box<Self::Result>> {
        if (self.condition)(injector, request_info) {
            self.inner.provide_owned_typed(injector, request_info)
        } else {
            Err(InjectError::ConditionsNotMet {
                service_info: ServiceInfo::of::<Self::Result>(),
            })
        }
    }
}

/// Defines a conversion into a conditional provider. This trait is
/// automatically implemented for all types that implement [`TypedProvider`].
pub trait WithCondition: TypedProvider {
    /// Creates a conditional provider. Conditional providers create their
    /// values only if their condition is met. If the condition is not met,
    /// then the provider is skipped.
    ///
    /// ## Example
    ///
    /// ```
    /// use runtime_injector::{Injector, IntoSingleton, Svc, WithCondition};
    ///
    /// #[derive(Default)]
    /// struct Foo;
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Foo::default.singleton().with_condition(|_, _| false));
    ///
    /// let injector = builder.build();
    /// let foo: Option<Svc<Foo>> = injector.get().unwrap();
    ///
    /// assert!(foo.is_none());
    /// ```
    #[must_use]
    fn with_condition<F>(self, condition: F) -> ConditionalProvider<Self, F>
    where
        F: Service + Fn(&Injector, &RequestInfo) -> bool;
}

impl<P> WithCondition for P
where
    P: TypedProvider,
{
    #[inline]
    fn with_condition<F>(self, condition: F) -> ConditionalProvider<Self, F>
    where
        F: Service + Fn(&Injector, &RequestInfo) -> bool,
    {
        ConditionalProvider {
            condition,
            inner: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::IntoSingleton;

    #[derive(Default)]
    struct Foo;

    /// When condition returns true, then a value is provided.
    #[test]
    fn test_condition_true() {
        let mut builder = Injector::builder();
        builder.provide(Foo::default.singleton().with_condition(|_, _| true));

        let injector = builder.build();
        let foo: Option<Svc<Foo>> = injector.get().unwrap();

        assert!(foo.is_some());
    }

    /// When condition returns true only once, then a value is provided only once.
    #[test]
    fn test_condition_true_once() {
        let mut builder = Injector::builder();
        let provided = Mutex::new(false);
        builder.provide(
            Foo::default.singleton()
                .with_condition(move |_, _| {
                    let mut provided = provided.lock().unwrap();
                    if *provided {
                        return false;
                    }
                    *provided = true;
                    true
                }),
        );

        // Create first value
        let injector = builder.build();
        let foo: Option<Svc<Foo>> = injector.get().unwrap();
        assert!(foo.is_some());

        // Create second value
        let foo: Option<Svc<Foo>> = injector.get().unwrap();
        assert!(foo.is_none());
    }

    /// When condition returns true after returning false, then a value is provided.
    #[test]
    fn test_condition_true_after_false() {
        let mut builder = Injector::builder();
        let provided = Mutex::new(false);
        builder.provide(
            Foo::default.singleton()
                .with_condition(move |_, _| {
                    let mut provided = provided.lock().unwrap();
                    if *provided {
                        return true;
                    }
                    *provided = true;
                    false
                }),
        );

        // Create first value
        let injector = builder.build();
        let foo: Option<Svc<Foo>> = injector.get().unwrap();
        assert!(foo.is_none());

        // Create second value
        let foo: Option<Svc<Foo>> = injector.get().unwrap();
        assert!(foo.is_some());
    }
}
