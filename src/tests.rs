#![allow(clippy::clippy::blacklisted_name)]

use crate::{
    constant, interface, InjectError, InjectResult, Injector, IntoSingleton,
    IntoTransient, ServiceInfo, Services, Svc, TypedProvider,
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

    let injector = builder.build();
    let _service: Svc<Svc1> = injector.get().unwrap();
}

#[test]
fn cant_make_svc1_when_no_provider() {
    let injector = Injector::builder().build();
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

    let injector = builder.build();
    let _service: Svc<Svc3> = injector.get().unwrap();
}

#[test]
fn cant_make_svc3_when_no_provider_for_dependency() {
    let mut builder = Injector::builder();
    builder.provide(Svc2::new.transient());
    builder.provide(Svc3::new.transient());

    let injector = builder.build();
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

    let injector = builder.build();
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

    let injector = builder.build();
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
    builder.provide(Svc1::default.transient().with_interface::<dyn Foo>());

    let injector = builder.build();
    let svc: Svc<dyn Foo> = injector.get().unwrap();

    assert_eq!(4, svc.bar());

    // Svc2
    let mut builder = Injector::builder();
    builder.provide(Svc1::default.transient());
    builder.provide(Svc2::new.transient().with_interface::<dyn Foo>());

    let injector = builder.build();
    let svc: Svc<dyn Foo> = injector.get().unwrap();

    assert_eq!(5, svc.bar());

    // Svc4
    let mut builder = Injector::builder();
    builder.provide(Svc1::default.transient());
    builder.provide(Svc2::new.transient().with_interface::<dyn Foo>());
    builder.provide(Svc4::new.transient());

    let injector = builder.build();
    let svc: Svc<Svc4> = injector.get().unwrap();

    assert_eq!(5, svc.foo.bar());
}

#[test]
fn multi_injection() {
    #[cfg(feature = "rc")]
    trait Foo {}

    #[cfg(feature = "arc")]
    trait Foo: Send + Sync {}

    impl Foo for Svc1 {}
    impl Foo for Svc2 {}
    impl Foo for Svc3 {}

    interface!(Foo = [Svc1, Svc2, Svc3]);

    let mut builder = Injector::builder();
    builder.provide(Svc1::default.transient().with_interface::<dyn Foo>());

    let injector = builder.build();
    let mut foos: Services<dyn Foo> = injector.get().unwrap();
    assert_eq!(1, foos.len());

    let foos: Vec<Svc<dyn Foo>> =
        foos.get_all().collect::<InjectResult<_>>().unwrap();
    assert_eq!(1, foos.len());
}

#[test]
fn injector_returns_error_on_cycles() {
    struct Foo(Svc<Bar>);
    impl Foo {
        fn new(bar: Svc<Bar>) -> Self {
            Foo(bar)
        }
    }

    struct Bar(Svc<Foo>);
    impl Bar {
        fn new(foo: Svc<Foo>) -> Self {
            Bar(foo)
        }
    }

    let mut builder = Injector::builder();
    builder.provide(Foo::new.singleton());
    builder.provide(Bar::new.singleton());

    let injector = builder.build();
    match injector.get::<Svc<Foo>>() {
        Err(InjectError::CycleDetected {
            service_info,
            cycle,
        }) if service_info == ServiceInfo::of::<Foo>() => {
            assert_eq!(3, cycle.len());
            assert_eq!(ServiceInfo::of::<Foo>(), cycle[0]);
            assert_eq!(ServiceInfo::of::<Bar>(), cycle[1]);
            assert_eq!(ServiceInfo::of::<Foo>(), cycle[2]);
        }
        Ok(_) => panic!("somehow created a Foo with a cyclic dependency"),
        Err(error) => Err(error).unwrap(),
    }
}
