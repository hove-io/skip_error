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

/// An iterator that ignore errors
pub struct SkipErrorIter<I, T, E>
where
    I: Iterator<Item = Result<T, E>>,
{
    inner: I,
    #[cfg(feature = "log")]
    log_level: Option<log::Level>,
}

impl<I, T, E> std::iter::Iterator for SkipErrorIter<I, T, E>
where
    I: Iterator<Item = Result<T, E>>,
    E: std::fmt::Display,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|result| match result {
            Ok(value) => Some(value),
            Err(_error) => {
                #[cfg(feature = "log")]
                if let Some(log_level) = self.log_level {
                    log::log!(log_level, "{}", _error);
                }
                self.next()
            }
        })
    }
}

/// Trait to extend any [`Iterator`] where the [`Iterator::Item`] is a [`Result`].
/// This allows to skip errors and keep only the `Ok()` values.
pub trait SkipError<I, T, E>: Sized
where
    I: Iterator<Item = Result<T, E>>,
{
    /// Skip all errors of the [`Result`] in the original [`Iterator`].
    /// This is essentially equivalent to `.flatten()`.
    ///
    /// ```edition2018
    /// use skip_error::SkipError;
    /// let v: Vec<usize> = vec![0,1,0,0,3]
    ///   .into_iter()
    ///   .map(|v|
    ///     if v == 0 {
    ///       Ok(0)
    ///     } else {
    ///       Err(format!("Boom on {}", v))
    ///     }
    ///   )
    ///   .skip_error()
    ///   .collect();
    /// assert_eq!(v, vec![0,0,0]);
    /// ```
    fn skip_error(self) -> SkipErrorIter<I, T, E>;

    /// Skip all errors of the [`Result`] in the original [`Iterator`].
    /// This also allows to log the errors, choosing which [`log::Level`] to use.
    ///
    /// ```edition2018
    /// use skip_error::SkipError;
    /// let v: Vec<usize> = vec![0,1,0,0,3]
    ///   .into_iter()
    ///   .map(|v|
    ///     if v == 0 {
    ///       Ok(0)
    ///     } else {
    ///       Err(format!("Boom on {}", v))
    ///     }
    ///   )
    ///   // Will log the following messages:
    ///   // - WARN: Boom on 1
    ///   // - WARN: Boom on 3
    ///   .skip_error_and_log(log::Level::Warn)
    ///   .collect();
    /// assert_eq!(v, vec![0,0,0]);
    /// ```
    #[cfg(feature = "log")]
    fn skip_error_and_log(self, log_level: log::Level) -> SkipErrorIter<I, T, E>;
}

impl<I, T, E> SkipError<I, T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
{
    fn skip_error(self) -> SkipErrorIter<I, T, E> {
        SkipErrorIter {
            inner: self,
            #[cfg(feature = "log")]
            log_level: None,
        }
    }
    #[cfg(feature = "log")]
    fn skip_error_and_log(self, log_level: log::Level) -> SkipErrorIter<I, T, E> {
        SkipErrorIter {
            inner: self,
            log_level: Some(log_level),
        }
    }
}
