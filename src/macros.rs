// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_log {
    (@helper $target:expr, $lvl:expr, $kvs:expr, $arg:literal $(,)?) => ({
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            const IS_LITERAL: bool = $crate::__private_api::is_literal($arg);

            if IS_LITERAL {
                $crate::__private_api::log(
                    &(
                        $crate::__private_api::module_path!(),
                        $crate::__private_api::file!(),
                    ),
                    $crate::__private_api::line!(),
                    $lvl,
                    $arg,
                    $target,
                    $kvs,
                );
            } else {
                $crate::__private_api::log(
                    &(
                        $crate::__private_api::module_path!(),
                        $crate::__private_api::file!(),
                    ),
                    $crate::__private_api::line!(),
                    $lvl,
                    $crate::__private_api::format_args!($arg),
                    $target,
                    $kvs,
                );
            }
        }
    });

    (@helper $target:expr, $lvl:expr, $kvs:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            $crate::__private_api::log(
                &(
                    $crate::__private_api::module_path!(),
                    $crate::__private_api::file!(),
                ),
                $crate::__private_api::line!(),
                $lvl,
                $crate::__private_api::format_args!($($arg)+),
                $target,
                $kvs,
            );
        }
    });

    // log!(target: "my_target", Level::Info, key1 = 42, key2 = true; "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($key:tt = $value:expr),+; $($arg:tt)+) => ($crate::__internal_log!(
        @helper
        $target,
        $lvl,
        &[
            $(($crate::__log_key!($key), &$value as &$crate::__private_api::LogKvValue)),+
        ] as &[_],
        $($arg)+
    ));

    // log!(target: "my_target", Level::Info, "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ($crate::__internal_log!(
        @helper
        $target,
        $lvl,
        $crate::__private_api::EmptyKVs,
        $($arg)+
    ));

    // log!(Level::Info, "a log event")
    ($lvl:expr, $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::TargetIsModulePath,
        $lvl,
        $($arg)+
    ));
}

/// The standard logging macro.
///
/// This macro will generically log with the specified `Level` and `format!`
/// based argument list.
///
/// # Examples
///
/// ```edition2018
/// use log::{log, Level};
///
/// # fn main() {
/// let data = (42, "Forty-two");
/// let private_data = "private";
///
/// log!(Level::Error, "Received errors: {}, {}", data.0, data.1);
/// log!(target: "app_events", Level::Warn, "App warning: {}, {}, {}",
///     data.0, data.1, private_data);
/// # }
/// ```
#[macro_export]
macro_rules! log {
    // log!(target: "my_target", Level::Info, key1 = 42, key2 = true; "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($key:tt = $value:expr),+; $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::identity::<&$crate::__private_api::str>($target),
        $crate::__private_api::identity::<$crate::Level>($lvl),
        $($key = $value),+;
        $($arg)+
    ));

    // log!(target: "my_target", Level::Info, "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::identity::<&$crate::__private_api::str>($target),
        $crate::__private_api::identity::<$crate::Level>($lvl),
        $($arg)+
    ));

    // log!(Level::Info, "a log event")
    ($lvl:expr, $($arg:tt)+) => ($crate::__internal_log!(
        $crate::__private_api::identity::<$crate::Level>($lvl),
        $($arg)+
    ));
}

/// Logs a message at the error level.
///
/// # Examples
///
/// ```edition2018
/// use log::error;
///
/// # fn main() {
/// let (err_info, port) = ("No connection", 22);
///
/// error!("Error: {} on port {}", err_info, port);
/// error!(target: "app_events", "App Error: {}, Port: {}", err_info, 22);
/// # }
/// ```
#[macro_export]
macro_rules! error {
    // error!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // error!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::identity::<&$crate::__private_api::str>($target),
        $crate::__private_api::StaticLevelError,
        $($arg)+
    ));

    // error!("a {} event", "log")
    ($($arg:tt)+) => ($crate::__internal_log!(
        $crate::__private_api::StaticLevelError,
        $($arg)+
    ))
}

/// Logs a message at the warn level.
///
/// # Examples
///
/// ```edition2018
/// use log::warn;
///
/// # fn main() {
/// let warn_description = "Invalid Input";
///
/// warn!("Warning! {}!", warn_description);
/// warn!(target: "input_events", "App received warning: {}", warn_description);
/// # }
/// ```
#[macro_export]
macro_rules! warn {
    // warn!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // warn!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::identity::<&$crate::__private_api::str>($target),
        $crate::__private_api::StaticLevelWarn,
        $($arg)+
    ));

    // warn!("a {} event", "log")
    ($($arg:tt)+) => ($crate::__internal_log!(
        $crate::__private_api::StaticLevelWarn,
        $($arg)+
    ))
}

/// Logs a message at the info level.
///
/// # Examples
///
/// ```edition2018
/// use log::info;
///
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// let conn_info = Connection { port: 40, speed: 3.20 };
///
/// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
/// info!(target: "connection_events", "Successful connection, port: {}, speed: {}",
///       conn_info.port, conn_info.speed);
/// # }
/// ```
#[macro_export]
macro_rules! info {
    // info!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // info!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::identity::<&$crate::__private_api::str>($target),
        $crate::__private_api::StaticLevelInfo,
        $($arg)+
    ));

    // info!("a {} event", "log")
    ($($arg:tt)+) => ($crate::__internal_log!(
        $crate::__private_api::StaticLevelInfo,
        $($arg)+
    ))
}

/// Logs a message at the debug level.
///
/// # Examples
///
/// ```edition2018
/// use log::debug;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!("New position: x: {}, y: {}", pos.x, pos.y);
/// debug!(target: "app_events", "New position: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
#[macro_export]
macro_rules! debug {
    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::identity::<&$crate::__private_api::str>($target),
        $crate::__private_api::StaticLevelDebug,
        $($arg)+
    ));

    // debug!("a {} event", "log")
    ($($arg:tt)+) => ($crate::__internal_log!(
        $crate::__private_api::StaticLevelDebug,
        $($arg)+
    ))
}

/// Logs a message at the trace level.
///
/// # Examples
///
/// ```edition2018
/// use log::trace;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
/// trace!(target: "app_events", "x is {} and y is {}",
///        if pos.x >= 0.0 { "positive" } else { "negative" },
///        if pos.y >= 0.0 { "positive" } else { "negative" });
/// # }
/// ```
#[macro_export]
macro_rules! trace {
    // trace!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // trace!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::__internal_log!(
        target: $crate::__private_api::identity::<&$crate::__private_api::str>($target),
        $crate::__private_api::StaticLevelTrace,
        $($arg)+
    ));

    // trace!("a {} event", "log")
    ($($arg:tt)+) => ($crate::__internal_log!(
        $crate::__private_api::StaticLevelTrace,
        $($arg)+
    ))
}

/// Determines if a message logged at the specified level in that module will
/// be logged.
///
/// This can be used to avoid expensive computation of log message arguments if
/// the message would be ignored anyway.
///
/// # Examples
///
/// ```edition2018
/// use log::Level::Debug;
/// use log::{debug, log_enabled};
///
/// # fn foo() {
/// if log_enabled!(Debug) {
///     let data = expensive_call();
///     debug!("expensive debug data: {} {}", data.x, data.y);
/// }
/// if log_enabled!(target: "Global", Debug) {
///    let data = expensive_call();
///    debug!(target: "Global", "expensive debug data: {} {}", data.x, data.y);
/// }
/// # }
/// # struct Data { x: u32, y: u32 }
/// # fn expensive_call() -> Data { Data { x: 0, y: 0 } }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! log_enabled {
    (target: $target:expr, $lvl:expr) => {{
        let lvl = $lvl;
        lvl <= $crate::STATIC_MAX_LEVEL
            && lvl <= $crate::max_level()
            && $crate::__private_api::enabled(lvl, $target)
    }};
    ($lvl:expr) => {
        $crate::log_enabled!(target: $crate::__private_api::module_path!(), $lvl)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_key {
    // key1 = 42
    ($($args:ident)*) => {
        $crate::__private_api::stringify!($($args)*)
    };
    // "key1" = 42
    ($($args:expr)*) => {
        $($args)*
    };
}
