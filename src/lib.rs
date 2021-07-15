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
//! Then you can use the macro [`skip_error!`] to write like this.
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
//! Or even better, use the trait [`SkipError`] that extends [`Iterator`] and do
//! the following (essentially equivalent to [`Iterator::flatten()`] but see
//! below for logging abilities).
//! ```edition2018
//! # #[macro_use]
//! # extern crate skip_error;
//! use skip_error::SkipError;
//! # fn main() {
//! let numbers: Vec<u32> = ["1", "2", "three", "4"]
//!   .into_iter()
//!   .map(|string_number| string_number.parse())
//!   .skip_error()
//!   .collect();
//! # }
//! ```
//!
//! # Logging
//!
//! If you want the error to be logged, you can use the feature `log` or the
//! feature `tracing` (see [Features](#features)). See [`skip_error_and_log!`]
//! and [`SkipError::skip_error_and_log()`] for more information.
//!
//! # Features
//!
//! - `log`: emit log message with the standard `std::log` macro. Disabled by
//! default.
//! - `tracing`: emit traces with the `tracing::trace` macro. Disabled
//! by default. If both `log` and `tracing` are enabled, then `log` will be
//! ignored since `tracing` is configured in a compatibility mode with standard
//! `log`.

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

/// `skip_error_and_log` returns the value of a `Result` or log and continues a
/// loop.
///
/// `skip_error_and_log` macro takes two parameters. The first argument is of
/// type `std::result::Result`. The second argument is anything that can be
/// turned into `log::Level` (feature `log`) or `tracing::Level` (feature
/// `tracing`) and defines the level to log to.  The macro returns the value if
/// `Result::Ok` and else, it logs the `Result::Error` and calls `continue`.
///
/// For example
/// ```edition2018
/// # #[macro_use]
/// # extern crate skip_error;
/// # fn main() {
/// # testing_logger::setup();
/// for string_number in &["1", "2", "three", "4"] {
#[cfg_attr(
    all(feature = "log", not(feature = "tracing")),
    doc = "  let number: u32 = skip_error_and_log!(string_number.parse(), log::Level::Warn);"
)]
#[cfg_attr(
    feature = "tracing",
    doc = "  let number: u32 = skip_error_and_log!(string_number.parse(), tracing::Level::WARN);"
)]
/// }
/// testing_logger::validate(|captured_logs| {
///   assert!(captured_logs[0].body.contains("invalid digit found in string"));
///   assert_eq!(captured_logs[0].level, log::Level::Warn);
/// });
/// # }
/// ```
#[macro_export]
#[cfg(any(feature = "log", feature = "tracing"))]
macro_rules! skip_error_and_log {
    ($result:expr, $log_level:expr) => {{
        match $result {
            Ok(value) => value,
            Err(error) => {
                $crate::__log!(error, $log_level);
                continue;
            }
        }
    }};
}

#[doc(hidden)]
#[macro_export]
#[cfg(all(feature = "log", not(feature = "tracing")))]
macro_rules! __log {
    ($error:expr, $log_level:expr) => {{
        log::log!(
            std::convert::Into::<log::Level>::into($log_level),
            "{}",
            $error
        );
    }};
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "tracing")]
macro_rules! __log {
    ($error:tt, $log_level:expr) => {{
        match std::convert::Into::<tracing::Level>::into($log_level) {
            tracing::Level::INFO => tracing::info!("{}", $error),
            tracing::Level::WARN => tracing::warn!("{}", $error),
            tracing::Level::ERROR => tracing::error!("{}", $error),
            tracing::Level::DEBUG => tracing::debug!("{}", $error),
            tracing::Level::TRACE => tracing::trace!("{}", $error),
        }
    }};
}

/// An iterator that ignore errors
pub struct SkipErrorIter<I, T, E>
where
    I: Iterator<Item = Result<T, E>>,
{
    inner: I,
    #[cfg(all(feature = "log", not(feature = "tracing")))]
    log_level: Option<log::Level>,
    #[cfg(feature = "tracing")]
    log_level: Option<tracing::Level>,
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
                #[cfg(any(feature = "log", feature = "tracing"))]
                if let Some(log_level) = self.log_level {
                    __log!(_error, log_level);
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

    /// Skip all errors of the [`Result`] in the original [`Iterator`].  This
    /// also allows to log the errors, choosing which [`log::Level`] to use.
    ///
    /// ```edition2018
    /// use skip_error::SkipError;
    /// # testing_logger::setup();
    /// let v: Vec<usize> = vec![0,1,0,0,3]
    ///   .into_iter()
    ///   .map(|v|
    ///     if v == 0 {
    ///       Ok(0)
    ///     } else {
    ///       Err(format!("Boom on {}", v))
    ///     }
    ///   )
    ///   .skip_error_and_log(log::Level::Warn)
    ///   .collect();
    /// assert_eq!(v, vec![0,0,0]);
    /// testing_logger::validate(|captured_logs| {
    ///   assert_eq!(captured_logs[0].level, log::Level::Warn);
    ///   assert_eq!(captured_logs[0].body, "Boom on 1");
    ///   assert_eq!(captured_logs[1].level, log::Level::Warn);
    ///   assert_eq!(captured_logs[1].body, "Boom on 3");
    /// });
    /// ```
    #[cfg(all(feature = "log", not(feature = "tracing")))]
    fn skip_error_and_log<L>(self, log_level: L) -> SkipErrorIter<I, T, E>
    where
        L: Into<log::Level>;
    ///
    /// Skip all errors of the [`Result`] in the original [`Iterator`].  This
    /// also allows to log the errors, choosing which [`tracing::Level`] to use.
    ///
    /// ```edition2018
    /// use skip_error::SkipError;
    /// # testing_logger::setup();
    /// let v: Vec<usize> = vec![0,1,0,0,3]
    ///   .into_iter()
    ///   .map(|v|
    ///     if v == 0 {
    ///       Ok(0)
    ///     } else {
    ///       Err(format!("Boom on {}", v))
    ///     }
    ///   )
    ///   .skip_error_and_log(tracing::Level::WARN)
    ///   .collect();
    /// assert_eq!(v, vec![0,0,0]);
    /// testing_logger::validate(|captured_logs| {
    ///   assert_eq!(captured_logs[0].level, log::Level::Warn);
    ///   assert_eq!(captured_logs[0].body, "Boom on 1 ");
    ///   assert_eq!(captured_logs[1].level, log::Level::Warn);
    ///   assert_eq!(captured_logs[1].body, "Boom on 3 ");
    /// });
    /// ```
    #[cfg(feature = "tracing")]
    fn skip_error_and_log<L>(self, trace_level: L) -> SkipErrorIter<I, T, E>
    where
        L: Into<tracing::Level>;
}

impl<I, T, E> SkipError<I, T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
{
    fn skip_error(self) -> SkipErrorIter<I, T, E> {
        SkipErrorIter {
            inner: self,
            #[cfg(any(feature = "log", feature = "tracing"))]
            log_level: None,
        }
    }
    #[cfg(all(feature = "log", not(feature = "tracing")))]
    fn skip_error_and_log<L>(self, log_level: L) -> SkipErrorIter<I, T, E>
    where
        L: Into<log::Level>,
    {
        SkipErrorIter {
            inner: self,
            log_level: Some(log_level.into()),
        }
    }
    #[cfg(feature = "tracing")]
    fn skip_error_and_log<L>(self, log_level: L) -> SkipErrorIter<I, T, E>
    where
        L: Into<tracing::Level>,
    {
        SkipErrorIter {
            inner: self,
            log_level: Some(log_level.into()),
        }
    }
}
