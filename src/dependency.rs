use crate::{Interface, ServiceInfo, Svc};

pub trait Dependencies: 'static {
    fn dependencies() -> Vec<ServiceInfo>;
}

macro_rules! impl_dependencies {
    () => {
        impl_dependencies!(@impl ());
    };
    ($first:ident $(, $rest:ident)*) => {
        impl_dependencies!(@impl ($first $(, $rest)*));
        impl_dependencies!($($rest),*);
    };

    (@impl ($($type_name:ident),*)) => {
        impl <$($type_name : ?Sized + Interface),*> Dependencies for ($(Svc<$type_name>,)*) {
            fn dependencies() -> Vec<ServiceInfo> {
                vec![$(ServiceInfo::of::<$type_name>()),*]
            }
        }
    };
}

impl_dependencies!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
