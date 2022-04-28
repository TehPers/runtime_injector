//! # Getting started
//!
//! Let's start with a simple application where we greet our users with a nice
//! message:
//!
//! ```
//! use std::io::stdin;
//!
//! fn main() {
//!     println!("What is your name?");
//!     let mut name = String::new();
//!     stdin().read_line(&mut name).unwrap();
//!     println!("Hello, {}! I hope you're having a wonderful day.", name);
//! }
//! ```
//!
//! Cool! We can now ask our user for their name and greet them! But what if we
//! want the user to be able to specify the name directly via command-line?
//!
//! ```
//! use std::{env::args, io::stdin};
//!
//! fn get_name() -> String {
//!     let name_parts: Vec<_> = args().skip(1).collect();
//!     if name_parts.is_empty() {
//!         println!("What is your name?");
//!         let mut name = String::new();
//!         stdin().read_line(&mut name).unwrap();
//!         name
//!     } else {
//!         name_parts.join(" ")
//!     }
//! }
//!
//! fn main() {
//!     let name = get_name();
//!     println!("Hello, {}! I hope you're having a wonderful day.", name);
//! }
//! ```
//!
//! Now our application lets users pass in their names directly, or if they
//! don't, we can prompt them to provide it to us! This is cool, but we can
//! still go further. Let's write some unit tests for our application so we can
//! make sure our code does what it's supposed to do, even if we change it in
//! the future. But wait, how do we verify that the output is correct? Well,
//! we need a way to check the output of our program somehow! Let's write an
//! abstraction for our output so we can check it it in our unit tests.
//!
//! ```
//! use std::{env::args, io::stdin};
//!
//! trait OutputWriter {
//!     fn write_output(&mut self, message: &str);
//! }
//!
//! struct ConsoleWriter;
//! impl OutputWriter for ConsoleWriter {
//!     fn write_output(&mut self, message: &str) {
//!         println!("{message}");
//!     }
//! }
//!
//! trait InputReader {
//!     fn read_line(&mut self) -> String;
//! }
//!
//! struct ConsoleReader;
//! impl InputReader for ConsoleReader {
//!     fn read_line(&mut self) -> String {
//!         let mut input = String::new();
//!         stdin().read_line(&mut input).unwrap();
//!         input
//!     }
//! }
//!
//! fn get_name<R: InputReader, W: OutputWriter>(
//!     args: &[String],
//!     reader: &mut R,
//!     writer: &mut W,
//! ) -> String {
//!     let name_parts: Vec<&str> =
//!         args.iter().skip(1).map(|s| s.as_str()).collect();
//!     if name_parts.is_empty() {
//!         writer.write_output("What is your name?");
//!         reader.read_line()
//!     } else {
//!         name_parts.join(" ")
//!     }
//! }
//!
//! fn main() {
//!     # // Let's actually verify the test passes
//!     # tests::name_is_correct();
//!     #
//!     let args: Vec<_> = args().collect();
//!     let mut reader = ConsoleReader;
//!     let mut writer = ConsoleWriter;
//!     let name = get_name(&args, &mut reader, &mut writer);
//!     writer.write_output(&format!(
//!         "Hello, {}! I hope you're having a wonderful day.",
//!         name
//!     ));
//! }
//!
//! // Verify our program works like we want it to
//! #[cfg(test)]
//! # mod _ignored_tests {}
//! mod tests {
//!     use super::*;
//!     use std::fmt::Write;
//!
//!     // Our mock writer so we can observe the output in tests
//!     struct MockWriter(pub String);
//!     impl OutputWriter for MockWriter {
//!         fn write_output(&mut self, message: &str) {
//!             writeln!(self.0, "{message}").unwrap();
//!         }
//!     }
//!
//!     // Our mock reader for ensuring anything that depends on it works
//!     struct MockReader(pub Option<String>);
//!     impl InputReader for MockReader {
//!         fn read_line(&mut self) -> String {
//!             self.0.take().unwrap()
//!         }
//!     }
//!
//!     // Let's make sure we're getting the correct name from the user
//!     #[test]
//!     # fn _ignored_test() {}
//!     # pub
//!     fn name_is_correct() {
//!         // Setup our mocked reader and writer
//!         let args = vec!["ignored".to_string()];
//!         let mut reader = MockReader(Some("John Smith".to_string()));
//!         let mut writer = MockWriter(String::new());
//!
//!         // Run the function we're testing
//!         let name = get_name(&args, &mut reader, &mut writer);
//!
//!         // Verify that we got the right name
//!         assert_eq!("John Smith", name);
//!         assert!(!writer.0.is_empty());
//!     }
//! }
//! ```
//!
//! Cool, now we have a simple unit test for our program. We can now verify
//! that it works automatically in our build pipelines, and we can be sure that
//! future improvements to our program won't break it!
//!
//! Speaking of future improvements, our users love the program! They keep
//! asking for more and more features to be added to it, though. There's a huge
//! group of users requesting that we give them control over the output
//! format, plus some users asking to be able to send these messages to people
//! on the internet! Not only that, but some people want the network requests
//! to be done via HTTPS and others want to use TCP. Woah, how in the world do
//! we configure our application to be able to do all this, yet still have the
//! ability to write unit tests for everything and understand what's going on?
//!
//! This is where dependency injection comes in. There is no way we could
//! possibly manage all the different ways of writing outputs, reading inputs,
//! configuring the greetings, and so on entirely on our own without our code
//! becoming huge and complex. We would end up with a tangled web of
//! dependencies between all the parts of our application, and it would quickly
//! become unmaintainable. Instead, let's rely on a container to manage our
//! dependencies for us so we don't need to think about that at all anymore.
//!
//! ```
//! use runtime_injector::{
//!     interface, Arg, Injector, InjectorBuilder, IntoSingleton, Service, Svc,
//!     TypedProvider, WithArg, WithInterface,
//! };
//! use std::io::stdin;
//!
//! // We need our trait to be a subtrait of `Service` so that we can use type
//! // erasure in our container later on. Also, if our services need to be
//! // thread-safe and we're using the "arc" feature for `runtime_injector`,
//! // then `Service` will automatically make `Send` + `Sync` required for us.
//! trait OutputWriter: Service {
//!     fn write_output(&mut self, message: &str);
//! }
//!
//! // We still want to be able to write to the console, but we need our output
//! // formatter to make sure we correctly format our output
//! struct ConsoleWriter(Svc<dyn OutputFormatter>);
//! impl OutputWriter for ConsoleWriter {
//!     fn write_output(&mut self, message: &str) {
//!         let message = self.0.fmt_message(message);
//!         println!("{message}");
//!     }
//! }
//!
//! // We also want to be able to send greetings via HTTP to a web service
//! struct HttpWriter(Svc<dyn OutputFormatter>);
//! impl OutputWriter for HttpWriter {
//!     fn write_output(&mut self, message: &str) {
//!         let _message = self.0.fmt_message(message);
//!         // ...
//!     }
//! }
//!
//! // Finally, we need to support TCP as well
//! struct TcpWriter(Svc<dyn OutputFormatter>);
//! impl OutputWriter for TcpWriter {
//!     fn write_output(&mut self, message: &str) {
//!         let _message = self.0.fmt_message(message);
//!         // ...
//!     }
//! }
//!
//! // We also need a way to format the messages
//! trait OutputFormatter: Service {
//!     fn fmt_message(&self, message: &str) -> String;
//! }
//!
//! // Our users want to be able to format them with custom formats!
//! struct UserFormatter(pub Arg<String>);
//! impl OutputFormatter for UserFormatter {
//!     fn fmt_message(&self, message: &str) -> String {
//!         format!("{message} formatted with {fmt}", fmt = self.0)
//!     }
//! }
//!
//! // Not all users want to use a custom format though, so we need a default
//! #[derive(Default)]
//! struct DefaultFormatter;
//! impl OutputFormatter for DefaultFormatter {
//!     fn fmt_message(&self, message: &str) -> String {
//!         message.into()
//!     }
//! }
//!
//! // Now let's bring over our reader implementations
//! trait InputReader: Service {
//!     fn read_line(&mut self) -> String;
//! }
//!
//! #[derive(Default)]
//! struct ConsoleReader;
//! impl InputReader for ConsoleReader {
//!     fn read_line(&mut self) -> String {
//!         let mut input = String::new();
//!         stdin().read_line(&mut input).unwrap();
//!         input
//!     }
//! }
//!
//! // Let's create an enum to help us configure our output writer too
//! #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
//! enum OutputType {
//!     Console,
//!     Https,
//!     Tcp,
//! }
//!
//! // Since we'll be relying on dependency injection, we need to provide our
//! // container a little more information by declaring our interfaces. First,
//! // we will declare `OutputWriter` to be an interface:
//! interface!(OutputWriter);
//!
//! // We also have the `OutputFormatter` interface:
//! interface!(OutputFormatter);
//!
//! // Finally, we have the `InputReader` interface:
//! interface!(InputReader);
//!
//! // We'll make a function to help us configure everything based on the
//! // configuration settings our user gave us
//! fn configure_services(
//!     user_format: Option<String>,
//!     output_type: OutputType,
//! ) -> InjectorBuilder {
//!     // Now how do we manage all these possible implementations? Let's rely
//!     // on a container to manage them for us!
//!     let mut builder = Injector::builder();
//!
//!     // Let's configure everything without worrying about the testing
//!     // environment. We want to configure our code to use the console reader
//!     // for reading input since we don't support any other input readers
//!     builder.provide(
//!         ConsoleReader::default
//!             .singleton()
//!             .with_interface::<dyn InputReader>(),
//!     );
//!
//!     // We want to determine the formatter based on whether the user
//!     // provided a format to us
//!     if let Some(user_format) = user_format {
//!         // Let's register the user formatter as our output formatter since
//!         // the user gave us a custom format to use
//!         builder.provide(
//!             UserFormatter
//!                 .singleton()
//!                 // We want to also pass the user's custom format to our
//!                 // service
//!                 .with_arg(user_format)
//!                 // We want our formatter to be requested through its
//!                 // interface
//!                 .with_interface::<dyn OutputFormatter>(),
//!         );
//!     } else {
//!         // The user didn't give us a custom format, so we'll use the
//!         // default output formatter instead
//!         builder.provide(
//!             DefaultFormatter::default
//!                 .singleton()
//!                 .with_interface::<dyn OutputFormatter>(),
//!         );
//!     }
//!
//!     // Finally, we need to decide how we're going to send our greetings to
//!     // people. We can use our helper enum for that here
//!     match output_type {
//!         OutputType::Console => {
//!             builder.provide(
//!                 ConsoleWriter
//!                     .singleton()
//!                     .with_interface::<dyn OutputWriter>(),
//!             );
//!         }
//!         OutputType::Https => {
//!             builder.provide(
//!                 HttpWriter
//!                     .singleton()
//!                     .with_interface::<dyn OutputWriter>(),
//!             );
//!         }
//!         OutputType::Tcp => {
//!             builder.provide(
//!                 TcpWriter.singleton().with_interface::<dyn OutputWriter>(),
//!             );
//!         }
//!     }
//!
//!     // Let's return our injector builder now
//!     builder
//! }
//!
//! fn main() {
//!     # // Let's actually verify the test passes
//!     # tests::console_output_is_formatted_before_being_written();
//!     #
//!     // We want the user to be able to configure the application here.
//!     // Normally, we'd use something like clap for this, but for the sake of
//!     // the example, we'll just hardcode the config
//!     let user_format = Some("Hello! Have a great day.".to_string());
//!     let output_type = OutputType::Console;
//!
//!     // With this, we have enough information to configure everything
//!     let builder = configure_services(user_format, output_type);
//!
//!     // Finally, we just need to construct our services so we can use them
//!     let injector = builder.build();
//!     let reader: Svc<dyn InputReader> = injector.get().unwrap();
//!     let writer: Svc<dyn OutputWriter> = injector.get().unwrap();
//!
//!     // Now we can write our application logic!
//!     // ...
//! }
//!
//! // Let's not forget about unit tests!
//! #[cfg(test)]
//! # mod _ignored_tests {}
//! mod tests {
//!     use super::*;
//!     use runtime_injector::{define_module, Injector, IntoSingleton, Svc};
#![cfg_attr(feature = "arc", doc = "     use std::sync::Mutex;")]
#![cfg_attr(feature = "rc", doc = "     use std::cell::RefCell;")]
//!
//!     // We may need to mock the output writer for testing to make sure we
//!     // are writing the correct message
#![cfg_attr(
    feature = "arc",
    doc = "     struct MockWriter(Svc<Mutex<String>>);"
)]
#![cfg_attr(
    feature = "rc",
    doc = "     struct MockWriter(Svc<RefCell<String>>);"
)]
//!     impl OutputWriter for MockWriter {
//!         fn write_output(&mut self, message: &str) {
//!             // We'll just track the message that was written
#![cfg_attr(
    feature = "arc",
    doc = "             let mut inner = self.0.lock().unwrap();"
)]
#![cfg_attr(
    feature = "rc",
    doc = "             let mut inner = self.0.borrow_mut();"
)]
//!             *inner = message.to_string();
//!         }
//!     }
//!
//!     struct MockReader(pub Arg<Option<String>>);
//!     impl InputReader for MockReader {
//!         fn read_line(&mut self) -> String {
//!              // We'll just return the message that was given to us
//!              self.0.take().unwrap()
//!         }
//!     }
//!
//!     #[test]
//!     # fn _ignored_test() {}
//!     # pub
//!     fn console_output_is_formatted_before_being_written() {
//!         // Let's make a custom module for testing just the console writer
//!         let module = define_module! {
//!             interfaces = {
//!                 dyn OutputFormatter = [
//!                     DefaultFormatter::default.singleton(),
//!                 ],
//!             },
//!             services = [
//!                 // We won't need to put this behind an interface this time
//!                 ConsoleWriter.singleton(),
//!             ],
//!         };
//!
//!         // We'll configure our injector now using the module we created
//!         let mut builder = Injector::builder();
//!         builder.add_module(module);
//!
//!         // Now we can test our console writer
//!         let injector = builder.build();
//!         let writer: Svc<ConsoleWriter> = injector.get().unwrap();
//!
//!         // ...
//!     }
//! }
//! ```
