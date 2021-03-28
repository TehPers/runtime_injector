#![allow(clippy::clippy::blacklisted_name)]

use crate::{
    constant, interface, InjectError, InjectResult, Injector, IntoSingleton,
    IntoTransient, ServiceInfo, Svc,
};
use std::sync::Mutex;

#[derive(Default)]
struct Svc1(pub i32);

struct Svc2 {
    pub dep1: Svc<Svc1>,
}

impl Svc2 {
    pub fn new(dep1: Svc<Svc1>) -> Self {
        Svc2 { dep1 }
    }
}

struct Svc3 {
    pub dep1: Svc<Svc1>,
    pub dep2: Svc<Svc2>,
}

impl Svc3 {
    pub fn new(dep1: Svc<Svc1>, dep2: Svc<Svc2>) -> Self {
        Svc3 { dep1, dep2 }
    }
}

#[test]
fn can_make_svc1() {
    let mut builder = Injector::builder();
    builder.provide(Svc1::default.transient());

    let mut injector = builder.build();
    let _service: Svc<Svc1> = injector.get().unwrap();
}

#[test]
fn cant_make_svc1_when_no_provider() {
    let mut injector = Injector::builder().build();
    let svc: InjectResult<Svc<Svc1>> = injector.get();
    match svc {
        Err(InjectError::MissingProvider { service_info })
            if service_info == ServiceInfo::of::<Svc1>() => {}
        Err(error) => Err(error).unwrap(),
        Ok(_) => unreachable!(),
    }

    let svc: Option<Svc<Svc1>> = injector.get().unwrap();
    match svc {
        None => {}
        Some(_) => panic!("service should not have been created"),
    }
}

#[test]
fn can_make_svc3() {
    let mut builder = Injector::builder();
    builder.provide(Svc1::default.transient());
    builder.provide(Svc2::new.transient());
    builder.provide(Svc3::new.transient());

    let mut injector = builder.build();
    let _service: Svc<Svc3> = injector.get().unwrap();
}

#[test]
fn cant_make_svc3_when_no_provider_for_dependency() {
    let mut builder = Injector::builder();
    builder.provide(Svc2::new.transient());
    builder.provide(Svc3::new.transient());

    let mut injector = builder.build();
    match injector.get::<Svc<Svc3>>() {
        Err(InjectError::MissingDependency {
            dependency_info, ..
        }) if dependency_info == ServiceInfo::of::<Svc1>() => {}
        Err(error) => Err(error).unwrap(),
        Ok(_) => unreachable!("service should not be able to be activated"),
    }
}

#[test]
fn singleton() {
    type Counter = Mutex<i32>;

    fn make_svc1(counter: Svc<Counter>) -> Svc1 {
        let mut counter = counter.lock().unwrap();
        *counter += 1;
        Svc1(*counter)
    }

    let mut builder = Injector::builder();
    builder.provide((|| Mutex::new(0)).singleton());
    builder.provide(make_svc1.transient());
    builder.provide(Svc2::new.transient());
    builder.provide(Svc3::new.transient());

    let mut injector = builder.build();
    let svc1: Svc<Svc1> = injector.get().unwrap();
    let svc2: Svc<Svc2> = injector.get().unwrap();
    let svc3: Svc<Svc3> = injector.get().unwrap();

    assert_ne!(svc1.0, svc2.dep1.0);
    assert_ne!(svc1.0, svc3.dep1.0);
    assert_ne!(svc2.dep1.0, svc3.dep1.0);
}

#[test]
fn constants() {
    type Counter = Mutex<i32>;

    fn make_svc1(counter: Svc<Counter>) -> Svc1 {
        let mut counter = counter.lock().unwrap();
        *counter += 1;
        Svc1(*counter)
    }

    let mut builder = Injector::builder();
    builder.provide(constant(Mutex::new(0)));
    builder.provide(make_svc1.transient());
    builder.provide(Svc2::new.transient());
    builder.provide(Svc3::new.transient());

    let mut injector = builder.build();
    let svc1: Svc<Svc1> = injector.get().unwrap();
    let svc2: Svc<Svc2> = injector.get().unwrap();
    let svc3: Svc<Svc3> = injector.get().unwrap();

    assert_ne!(svc1.0, svc2.dep1.0);
    assert_ne!(svc1.0, svc3.dep1.0);
    assert_ne!(svc2.dep1.0, svc3.dep1.0);
}

#[test]
fn interfaces() {
    #[cfg(feature = "rc")]
    pub trait Foo {
        fn bar(&self) -> i32;
    }

    #[cfg(feature = "arc")]
    pub trait Foo: Send + Sync {
        fn bar(&self) -> i32;
    }

    interface!(
        Foo = [
            Svc1,
            #[cfg(test)]
            Svc2,
            #[cfg(not(test))]
            Svc3,
        ]
    );

    impl Foo for Svc1 {
        fn bar(&self) -> i32 {
            4
        }
    }

    impl Foo for Svc2 {
        fn bar(&self) -> i32 {
            5
        }
    }

    struct Svc4 {
        pub foo: Svc<dyn Foo>,
    }

    impl Svc4 {
        pub fn new(foo: Svc<dyn Foo>) -> Self {
            Svc4 { foo }
        }
    }

    // Svc1
    let mut builder = Injector::builder();
    builder.provide(Svc1::default.transient());
    builder.provide(Svc2::new.transient());
    builder.implement::<dyn Foo, Svc1>();
    builder.provide(Svc4::new.transient());

    let mut injector = builder.build();
    let svc: Svc<dyn Foo> = injector.get().unwrap();

    assert_eq!(4, svc.bar());

    // Svc2
    let mut builder = Injector::builder();
    builder.provide(Svc1::default.transient());
    builder.provide(Svc2::new.transient());
    builder.implement::<dyn Foo, Svc2>();

    let mut injector = builder.build();
    let svc: Svc<dyn Foo> = injector.get().unwrap();

    assert_eq!(5, svc.bar());
}

#[test]
fn a() {
    trait Foo: Send + Sync {}
    interface!(Foo = [Bar]);

    #[derive(Default)]
    struct Bar;
    impl Foo for Bar {}

    let mut builder = Injector::builder();
    builder.provide(Bar::default.singleton());
    builder.implement::<dyn Foo, Bar>();

    let mut injector = builder.build();
    let _bar: Svc<dyn Foo> = injector.get().unwrap();
}
