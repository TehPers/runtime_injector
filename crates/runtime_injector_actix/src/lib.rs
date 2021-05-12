//! Utility library for injecting dependencies into actix-web applications.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::needless_doctest_main,
    clippy::needless_pass_by_value
)]

pub use runtime_injector::*;

mod service;

pub use service::*;
