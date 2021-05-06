use crate::{Provider, ProviderMap};

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
}

/// Defines a new module using a domain specific language.
///
/// # Example
///
/// ```
/// use runtime_injector::{define_module, interface, IntoSingleton, IntoTransient, Svc, Injector};
///
/// struct Foo();
/// struct Bar();
/// struct Baz(Vec<Svc<dyn Fooable>>);
///
/// trait Fooable: Send + Sync {}
/// impl Fooable for Foo {}
/// impl Fooable for Bar {}
/// interface! {
///     Fooable = [Foo, Bar]
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
/// };
///
/// let mut builder = Injector::builder();
/// builder.add_module(module);
///
/// let injector = builder.build();
/// let baz: Svc<Baz> = injector.get().unwrap();
/// assert_eq!(2, baz.0.len());
/// ```
#[macro_export]
macro_rules! define_module {
    {
        $($key:tt = $value:tt),*
        $(,)?
    } => {
        {
            #[allow(unused_mut)]
            let mut module = <$crate::Module as ::std::default::Default>::default();
            $($crate::define_module!(@provide module, $key = $value);)*
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
        ),*
    };
}
