//! Utility library for injecting dependencies into actix-web applications.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value
)]

pub use runtime_injector::*;

mod service;

pub use service::*;
