#![deny(missing_docs)]

//! This crate provides a single macro to help skipping a error in a loop,
//! possibly logging it.
//!
//! For example, imagine you have some code like this.
//! ```edition2018
//! for string_number in &["1", "2", "three", "4"] {
//!   let number: u32 = match string_number.parse() {
//!     Ok(n) => n,
//!     Err(e) => continue,
//!   };
//! }
//! ```
//!
//! Then you can use the macro `skip_error!` to write like this.
//! ```edition2018
//! # #[macro_use]
//! # extern crate skip_error;
//! # fn main() {
//! for string_number in &["1", "2", "three", "4"] {
//!   let number: u32 = skip_error!(string_number.parse());
//! }
//! # }
//! ```
//!
//! If you want the error to be logged, you can use the feature `log`. The
//! logging will be done in WARN level with the standard logging interface
//! provided by [`log`](https://crates.io/crates/log).

/// `skip_error` returns the value of a `Result` or continues a loop.
///
/// `skip_error` macro takes one parameter of type `std::result::Result`. It
/// returns the value if `Result::Ok` or else, it calls `continue` and ignore
/// the `Result::Error`.
///
/// For example
/// ```edition2018
/// # #[macro_use]
/// # extern crate skip_error;
/// # fn main() {
/// for string_number in &["1", "2", "three", "4"] {
///   let number: u32 = skip_error!(string_number.parse());
/// }
/// # }
/// ```
#[macro_export]
macro_rules! skip_error {
    ($result:expr) => {{
        match $result {
            Ok(value) => value,
            Err(error) => {
                continue;
            }
        }
    }};
}

/// `skip_error_and_log` returns the value of a `Result` or log and continues a loop.
///
/// `skip_error_and_log` macro takes two parameters. The first argument is of
/// type `std::result::Result`. The second argument take the `log::Level` to use
/// for the logging.  The macro returns the value if `Result::Ok` and else, it
/// logs the `Result::Error` and calls `continue`.
///
/// For example
/// ```edition2018
/// # #[macro_use]
/// # extern crate skip_error;
/// # fn main() {
/// for string_number in &["1", "2", "three", "4"] {
///   let number: u32 = skip_error_and_log!(string_number.parse(), log::Level::Warn);
/// }
/// # }
/// ```
#[macro_export]
#[cfg(feature = "log")]
macro_rules! skip_error_and_log {
    ($result:expr, $log_level:expr) => {{
        match $result {
            Ok(value) => value,
            Err(error) => {
                log::log!($log_level, "{}", error);
                continue;
            }
        }
    }};
}
