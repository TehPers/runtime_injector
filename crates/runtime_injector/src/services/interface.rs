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
    S: ?Sized + Service,
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
///
/// ## Generic interfaces
///
/// Traits with generic type parameters, associated types, or `where` clauses
/// can be marked as interfaces as well. For generic type parameters, just use
/// the familiar `<T1, T2, ...>` syntax. Then for associated types, use `assoc
/// T1, T2`. Finally, for `where` clauses, use `where T1: Trait, T2: Trait`.
/// The order is important (type parameters, then associated types, then
/// `where` clauses).
///
/// *This syntax is based on the syntax used by
/// [downcast_rs](https://docs.rs/downcast_rs).*
///
/// ```
/// use runtime_injector::{
///     constant, interface, Injector, Service, Svc, WithInterface,
/// };
/// use std::fmt::Debug;
///
/// trait DataSource<T>: Service
/// where
///     T: Debug,
/// {
///     type Id;
///     fn get(&self, id: Self::Id) -> Option<T>;
/// }
/// interface!(DataSource<T> assoc Id where T: Debug);
///
/// #[derive(Debug)]
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// struct UserDataSource;
/// impl DataSource<User> for UserDataSource {
///     type Id = u32;
///
///     fn get(&self, id: Self::Id) -> Option<User> {
///         if id == 1 {
///             Some(User {
///                 id,
///                 name: "example".to_string(),
///             })
///         } else {
///             None
///         }
///     }
/// }
///
/// let mut builder = Injector::builder();
/// builder.provide(
///     constant(UserDataSource)
///         .with_interface::<dyn DataSource<User, Id = u32>>(),
/// );
///
/// let injector = builder.build();
/// let user_data_source: Svc<dyn DataSource<User, Id = u32>> =
///     injector.get().unwrap();
/// assert!(user_data_source.get(0).is_none());
/// let user = user_data_source.get(1).unwrap();
/// assert_eq!("example", user.name);
/// ```
#[macro_export]
macro_rules! interface {
    (
        $interface:ident
        $(<$($types:ident),* $(,)?>)?
        $(assoc $($assoc:ident),* $(,)?)?
        $(where $($where:tt)*)?
    ) => {
        $crate::interface! {
            @impl
            interface = $interface,
            types = [$($($types),*)?],
            assoc = [$($($assoc),*)?],
            where = [$($($where)*)?],
            unused_name = __RUNTIME_INJECTOR_T,
        }
    };
    {
        @impl
        interface = $interface:ident,
        types = [$($types:ident),*],
        assoc = [$($assoc:ident),*],
        where = [$($where:tt)*],
        unused_name = $unused_name:ident,
    } => {
        impl<
            $($types,)* $($assoc,)*
        > $crate::Interface for dyn $interface<
            $($types,)*
            $($assoc = $assoc,)*
        >
        where
            Self: $crate::Service,
            $($where)*
        {}

        impl<
            $($types,)*
            $($assoc,)*
        > $crate::InterfaceFor<Self> for dyn $interface<
            $($types,)*
            $($assoc = $assoc,)*
        >
        where
            Self: $crate::Interface,
            $($where)*
        {
            fn from_svc(service: $crate::Svc<Self>) -> $crate::Svc<Self> {
                service
            }

            fn from_owned_svc(
                service: ::std::boxed::Box<Self>,
            ) -> ::std::boxed::Box<Self> {
                service
            }
        }

        #[allow(non_camel_case_types)]
        impl<
            $($types,)*
            $($assoc,)*
            $unused_name,
        > $crate::InterfaceFor<$unused_name> for dyn $interface<
            $($types,)*
            $($assoc = $assoc,)*
        >
        where
            Self: $crate::Interface,
            $unused_name: $interface<
                $($types,)*
                $($assoc = $assoc,)*
            >,
            $($where)*
        {
            fn from_svc(service: $crate::Svc<$unused_name>) -> $crate::Svc<Self> {
                service
            }

            fn from_owned_svc(
                service: ::std::boxed::Box<$unused_name>,
            ) -> ::std::boxed::Box<Self> {
                service
            }
        }

        impl<
            $($types,)*
            $($assoc,)*
        > $crate::FromProvider for dyn $interface<
            $($types,)*
            $($assoc = $assoc,)*
        >
        where
            Self: $crate::Interface,
            $($where)*
        {
            type Interface = Self;

            fn should_provide(
                _provider: &dyn $crate::Provider<Interface = Self::Interface>,
            ) -> bool {
                true
            }

            fn from_interface(
                provided: $crate::Svc<Self::Interface>,
            ) -> $crate::InjectResult<$crate::Svc<Self>> {
                ::std::result::Result::Ok(provided)
            }

            fn from_interface_owned(
                provided: ::std::boxed::Box<Self::Interface>,
            ) -> $crate::InjectResult<::std::boxed::Box<Self>> {
                ::std::result::Result::Ok(provided)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constant, Injector, WithInterface};
    use std::marker::PhantomData;

    #[test]
    fn a_impl_can_be_provided_as_a() {
        trait A: Service {}
        interface!(A);

        struct AImpl;
        impl A for AImpl {}

        let mut builder = Injector::builder();
        builder.provide(constant(AImpl).with_interface::<dyn A>());

        let injector = builder.build();
        let _a: Svc<dyn A> = injector.get().unwrap();
    }

    #[test]
    fn b_impl_can_be_provided_as_b() {
        trait B<T>: Service {
            fn get(&self) -> &T;
        }
        interface!(B<T>);

        struct BImpl<T>(T)
        where
            T: Service;
        impl<T> B<T> for BImpl<T>
        where
            T: Service,
        {
            fn get(&self) -> &T {
                &self.0
            }
        }

        let mut builder = Injector::builder();
        builder.provide(constant(BImpl(42)).with_interface::<dyn B<u32>>());
        builder.provide(
            constant(BImpl("hello, world!"))
                .with_interface::<dyn B<&'static str>>(),
        );

        let injector = builder.build();
        let b: Svc<dyn B<u32>> = injector.get().unwrap();
        assert_eq!(42, *b.get());

        let b: Svc<dyn B<&'static str>> = injector.get().unwrap();
        assert_eq!("hello, world!", *b.get());
    }

    #[test]
    fn c_impl_can_be_provided_as_c() {
        trait C<In>: Service
        where
            Self::Out: Service,
        {
            type Out;
            fn get(&self, value: In) -> Self::Out;
        }
        interface!(C<In> assoc Out where Out: Service);

        struct CImpl<In, Out, F>(F, PhantomData<fn(In) -> Out>)
        where
            In: Service,
            Out: Service,
            F: Service + Fn(In) -> Out;
        impl<In, Out, F> C<In> for CImpl<In, Out, F>
        where
            In: Service,
            Out: Service,
            F: Service + Fn(In) -> Out,
        {
            type Out = Out;

            fn get(&self, value: In) -> Self::Out {
                (self.0)(value)
            }
        }

        let mut builder = Injector::builder();
        builder.provide(
            constant(CImpl(|x| x + 1, PhantomData))
                .with_interface::<dyn C<u32, Out = u32>>(),
        );
        builder.provide(
            constant(CImpl(|x| format!("input: {x}"), PhantomData))
                .with_interface::<dyn C<&'static str, Out = String>>(),
        );

        let injector = builder.build();
        let c: Svc<dyn C<u32, Out = u32>> = injector.get().unwrap();
        assert_eq!(43, c.get(42));

        let c: Svc<dyn C<&'static str, Out = String>> = injector.get().unwrap();
        assert_eq!("input: 42", c.get("42"));
    }
}
