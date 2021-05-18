use crate::{Provider, ProviderMap, RequestParameter};
use std::collections::HashMap;

/// A collection of providers that can be added all at once to an
/// [`InjectorBuilder`](crate::InjectorBuilder). Modules can be used to group
/// together related services and configure the injector in pieces rather than
/// all at once.
///
/// For creating a module easily via a domain specific language, see
/// [`define_module!`].
#[derive(Default)]
pub struct Module {
    pub(crate) providers: ProviderMap,
    pub(crate) parameters: HashMap<String, Box<dyn RequestParameter>>,
}

impl Module {
    /// Assigns the provider for a service type. Multiple providers can be
    /// registered for a service.
    #[allow(clippy::missing_panics_doc)]
    pub fn provide<P: Provider>(&mut self, provider: P) {
        // Should never panic
        self.providers
            .entry(provider.result())
            .or_insert_with(|| Some(Vec::new()))
            .as_mut()
            .unwrap()
            .push(Box::new(provider));
    }

    /// Sets the of a value request parameter for requests made by the injector
    /// this module is added to. If a parameter has already been set to a
    /// value in this module, then that value is returned.
    pub fn insert_parameter(
        &mut self,
        key: &str,
        value: impl RequestParameter,
    ) -> Option<Box<dyn RequestParameter>> {
        self.parameters.insert(key.to_owned(), Box::new(value))
    }

    /// Removes and returns the value of a parameter if it has been set.
    pub fn remove_parameter(
        &mut self,
        key: &str,
    ) -> Option<Box<dyn RequestParameter>> {
        self.parameters.remove(key)
    }
}

/// Defines a new module using a domain specific language.
///
/// ## Example
///
/// ```
/// use runtime_injector::{
///     define_module, interface, Arg, Injector, IntoSingleton, IntoTransient,
///     Service, Svc,
/// };
///
/// struct Foo(Arg<i32>);
/// struct Bar();
/// struct Baz(Vec<Svc<dyn Fooable>>);
/// #[cfg(test)]
/// struct Quux();
///
/// trait Fooable: Service {}
/// impl Fooable for Foo {}
/// impl Fooable for Bar {}
/// interface! {
///     dyn Fooable = [
///         Foo,
///         Bar,
///         #[cfg(test)]
///         Quux,
///     ]
/// };
///
/// let module = define_module! {
///     services = [
///         Baz.singleton(),
///     ],
///     interfaces = {
///         dyn Fooable = [
///             Foo.singleton(),
///             Bar.singleton(),
///         ],
///     },
///     arguments = {
///         Foo = [12i32],
///     },
///
///     // If there are multiple interface or service definitions, they are
///     // merged together. This means we can have providers registered only in
///     // certain environments.
///     #[cfg(test)]
///     interfaces = {
///         dyn Fooable = [
///             Quux.singleton(),
///         ],
///     },
/// };
///
/// let mut builder = Injector::builder();
/// builder.add_module(module);
///
/// let injector = builder.build();
/// let baz: Svc<Baz> = injector.get().unwrap();
///
/// #[cfg(not(test))]
/// assert_eq!(2, baz.0.len());
/// #[cfg(test)]
/// assert_eq!(3, baz.0.len());
/// ```
#[macro_export]
macro_rules! define_module {
    {
        $(
            $(#[$($attr:meta),*])*
            $key:ident = $value:tt
        ),*
        $(,)?
    } => {
        {
            #[allow(unused_mut)]
            let mut module = <$crate::Module as ::std::default::Default>::default();
            $(
                $(#[$($attr),*])*
                $crate::define_module!(@provide &mut module, $key = $value);
            )*
            module
        }
    };
    (
        @provide $module:expr,
        services = [
            $($service:expr),*
            $(,)?
        ]
    ) => {
        $($module.provide($service);)*
    };
    (
        @provide $module:expr,
        interfaces = {
            $($interface:ty = [
                $($implementation:expr),*
                $(,)?
            ]),*
            $(,)?
        }
    ) => {
        $(
            $($module.provide($crate::TypedProvider::with_interface::<$interface>($implementation));)*
        )*
    };
    (
        @provide $module:expr,
        arguments = {
            $($service:ty = [
                $($arg:expr),*
                $(,)?
            ]),*
            $(,)?
        }
    ) => {
        $(
            $($crate::WithArg::with_arg::<$service, _>($module, $arg);)*
        )*
    };
}
